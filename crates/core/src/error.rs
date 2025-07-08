use autoagents::core_error::Error as CoreError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    AutoAgentsError(#[from] CoreError),
}
