// Copyright 2022 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt::Debug;
use std::io::ErrorKind;
use std::io::Result;
use std::sync::Arc;
use std::thread::sleep;

use async_trait::async_trait;
use backon::Backoff;
use backon::Retryable;

use crate::multipart::ObjectPart;
use crate::ops::OpAbortMultipart;
use crate::ops::OpCompleteMultipart;
use crate::ops::OpCreate;
use crate::ops::OpCreateMultipart;
use crate::ops::OpDelete;
use crate::ops::OpList;
use crate::ops::OpPresign;
use crate::ops::OpRead;
use crate::ops::OpStat;
use crate::ops::OpWrite;
use crate::ops::OpWriteMultipart;
use crate::ops::PresignedRequest;
use crate::Accessor;
use crate::AccessorMetadata;
use crate::BlockingBytesReader;
use crate::BytesReader;
use crate::DirIterator;
use crate::DirStreamer;
use crate::Layer;
use crate::ObjectMetadata;

/// RetryLayer will add retry for OpenDAL.
///
/// # Examples
///
/// ```
/// use anyhow::Result;
/// use backon::ExponentialBackoff;
/// use opendal::layers::RetryLayer;
/// use opendal::Operator;
/// use opendal::Scheme;
///
/// let _ = Operator::from_env(Scheme::Fs)
///     .expect("must init")
///     .layer(RetryLayer::new(ExponentialBackoff::default()));
/// ```
pub struct RetryLayer<B: Backoff + Send + Sync + Debug + 'static>(B);

impl<B> RetryLayer<B>
where
    B: Backoff + Send + Sync + Debug + 'static,
{
    /// Create a new retry layer.
    /// # Examples
    ///
    /// ```
    /// use anyhow::Result;
    /// use backon::ExponentialBackoff;
    /// use opendal::layers::RetryLayer;
    /// use opendal::Operator;
    /// use opendal::Scheme;
    ///
    /// let _ = Operator::from_env(Scheme::Fs)
    ///     .expect("must init")
    ///     .layer(RetryLayer::new(ExponentialBackoff::default()));
    /// ```
    pub fn new(b: B) -> Self {
        Self(b)
    }
}

impl<B> Layer for RetryLayer<B>
where
    B: Backoff + Send + Sync + Debug + 'static,
{
    fn layer(&self, inner: Arc<dyn Accessor>) -> Arc<dyn Accessor> {
        Arc::new(RetryAccessor {
            inner,
            backoff: self.0.clone(),
        })
    }
}

#[derive(Debug)]
struct RetryAccessor<B: Backoff + Debug + Send + Sync> {
    inner: Arc<dyn Accessor>,
    backoff: B,
}

#[async_trait]
impl<B> Accessor for RetryAccessor<B>
where
    B: Backoff + Debug + Send + Sync,
{
    fn metadata(&self) -> AccessorMetadata {
        self.inner.metadata()
    }

    async fn create(&self, args: &OpCreate) -> Result<()> {
        { || self.inner.create(args) }
            .retry(self.backoff.clone())
            .when(|e| e.kind() == ErrorKind::Interrupted)
            .await
    }
    async fn read(&self, args: &OpRead) -> Result<BytesReader> {
        { || self.inner.read(args) }
            .retry(self.backoff.clone())
            .when(|e| e.kind() == ErrorKind::Interrupted)
            .await
    }
    async fn write(&self, args: &OpWrite, r: BytesReader) -> Result<u64> {
        // Write can't retry, until can reset this reader.
        self.inner.write(args, r).await
    }
    async fn stat(&self, args: &OpStat) -> Result<ObjectMetadata> {
        { || self.inner.stat(args) }
            .retry(self.backoff.clone())
            .when(|e| e.kind() == ErrorKind::Interrupted)
            .await
    }
    async fn delete(&self, args: &OpDelete) -> Result<()> {
        { || self.inner.delete(args) }
            .retry(self.backoff.clone())
            .when(|e| e.kind() == ErrorKind::Interrupted)
            .await
    }
    async fn list(&self, args: &OpList) -> Result<DirStreamer> {
        { || self.inner.list(args) }
            .retry(self.backoff.clone())
            .when(|e| e.kind() == ErrorKind::Interrupted)
            .await
    }

    fn presign(&self, args: &OpPresign) -> Result<PresignedRequest> {
        self.inner.presign(args)
    }

    async fn create_multipart(&self, args: &OpCreateMultipart) -> Result<String> {
        { || self.inner.create_multipart(args) }
            .retry(self.backoff.clone())
            .when(|e| e.kind() == ErrorKind::Interrupted)
            .await
    }
    async fn write_multipart(&self, args: &OpWriteMultipart, r: BytesReader) -> Result<ObjectPart> {
        // Write can't retry, until can reset this reader.
        self.inner.write_multipart(args, r).await
    }
    async fn complete_multipart(&self, args: &OpCompleteMultipart) -> Result<()> {
        { || self.inner.complete_multipart(args) }
            .retry(self.backoff.clone())
            .when(|e| e.kind() == ErrorKind::Interrupted)
            .await
    }
    async fn abort_multipart(&self, args: &OpAbortMultipart) -> Result<()> {
        { || self.inner.abort_multipart(args) }
            .retry(self.backoff.clone())
            .when(|e| e.kind() == ErrorKind::Interrupted)
            .await
    }

    fn blocking_create(&self, args: &OpCreate) -> Result<()> {
        let retry = self.backoff.clone();

        let mut e = None;

        for dur in retry {
            let res = self.inner.blocking_create(args);

            match res {
                Ok(v) => return Ok(v),
                Err(err) => {
                    let kind = err.kind();
                    e = Some(err);

                    if kind == ErrorKind::Interrupted {
                        sleep(dur);
                        continue;
                    } else {
                        return Err(e.unwrap());
                    }
                }
            }
        }

        Err(e.unwrap())
    }

    fn blocking_read(&self, args: &OpRead) -> Result<BlockingBytesReader> {
        let retry = self.backoff.clone();

        let mut e = None;

        for dur in retry {
            let res = self.inner.blocking_read(args);

            match res {
                Ok(v) => return Ok(v),
                Err(err) => {
                    let kind = err.kind();
                    e = Some(err);

                    if kind == ErrorKind::Interrupted {
                        sleep(dur);
                        continue;
                    } else {
                        return Err(e.unwrap());
                    }
                }
            }
        }

        Err(e.unwrap())
    }

    fn blocking_write(&self, args: &OpWrite, r: BlockingBytesReader) -> Result<u64> {
        self.inner.blocking_write(args, r)
    }

    fn blocking_stat(&self, args: &OpStat) -> Result<ObjectMetadata> {
        let retry = self.backoff.clone();

        let mut e = None;

        for dur in retry {
            let res = self.inner.blocking_stat(args);

            match res {
                Ok(v) => return Ok(v),
                Err(err) => {
                    let kind = err.kind();
                    e = Some(err);

                    if kind == ErrorKind::Interrupted {
                        sleep(dur);
                        continue;
                    } else {
                        return Err(e.unwrap());
                    }
                }
            }
        }

        Err(e.unwrap())
    }

    fn blocking_delete(&self, args: &OpDelete) -> Result<()> {
        let retry = self.backoff.clone();

        let mut e = None;

        for dur in retry {
            let res = self.inner.blocking_delete(args);

            match res {
                Ok(v) => return Ok(v),
                Err(err) => {
                    let kind = err.kind();
                    e = Some(err);

                    if kind == ErrorKind::Interrupted {
                        sleep(dur);
                        continue;
                    } else {
                        return Err(e.unwrap());
                    }
                }
            }
        }

        Err(e.unwrap())
    }

    fn blocking_list(&self, args: &OpList) -> Result<DirIterator> {
        let retry = self.backoff.clone();

        let mut e = None;

        for dur in retry {
            let res = self.inner.blocking_list(args);

            match res {
                Ok(v) => return Ok(v),
                Err(err) => {
                    let kind = err.kind();
                    e = Some(err);

                    if kind == ErrorKind::Interrupted {
                        sleep(dur);
                        continue;
                    } else {
                        return Err(e.unwrap());
                    }
                }
            }
        }

        Err(e.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::sync::Arc;
    use std::time::Duration;

    use anyhow::anyhow;
    use async_trait::async_trait;
    use backon::ConstantBackoff;
    use tokio::sync::Mutex;

    use crate::error::other;
    use crate::layers::RetryLayer;
    use crate::ops::OpRead;
    use crate::Accessor;
    use crate::BytesReader;
    use crate::Operator;

    #[derive(Debug, Clone, Default)]
    struct MockService {
        attempt: Arc<Mutex<usize>>,
    }

    #[async_trait]
    impl Accessor for MockService {
        async fn read(&self, args: &OpRead) -> io::Result<BytesReader> {
            let mut attempt = self.attempt.lock().await;
            *attempt += 1;

            match args.path() {
                "retryable_error" => Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    anyhow!("retryable_error"),
                )),
                _ => Err(other(anyhow!("not_retryable_error"))),
            }
        }
    }

    #[tokio::test]
    async fn test_retry_retryable_error() -> anyhow::Result<()> {
        let srv = Arc::new(MockService::default());

        let backoff = ConstantBackoff::default()
            .with_delay(Duration::from_micros(1))
            .with_max_times(10);
        let op = Operator::new(srv.clone()).layer(RetryLayer::new(backoff));

        let result = op.object("retryable_error").read().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "retryable_error");
        // The error is retryable, we should request it 1 + 10 times.
        assert_eq!(*srv.attempt.lock().await, 11);

        Ok(())
    }

    #[tokio::test]
    async fn test_retry_not_retryable_error() -> anyhow::Result<()> {
        let srv = Arc::new(MockService::default());

        let backoff = ConstantBackoff::default();
        let op = Operator::new(srv.clone()).layer(RetryLayer::new(backoff));

        let result = op.object("not_retryable_error").read().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "not_retryable_error");
        // The error is not retryable, we should only request it once.
        assert_eq!(*srv.attempt.lock().await, 1);

        Ok(())
    }
}
