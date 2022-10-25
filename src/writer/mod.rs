use async_trait::async_trait;

#[async_trait]
pub trait Writer {
    // TODO: do after finish request and response
    async fn write(mut self);
}