use async_trait::async_trait;

#[async_trait]
pub trait Writer {
    async fn write(mut self);
}