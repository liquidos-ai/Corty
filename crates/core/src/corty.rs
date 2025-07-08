use crate::agent::CodingAgent;
use crate::error::Error;
use autoagents::core::agent::{AgentBuilder, RunnableAgent};
use autoagents::core::environment::Environment;
use autoagents::core::memory::SlidingWindowMemory;
use autoagents::core::protocol::AgentID;
use autoagents::llm::backends::openai::OpenAI;
use autoagents::llm::builder::LLMBuilder;
use std::sync::Arc;
use uuid::Uuid;

pub struct Corty {
    environment: Environment,
}

impl Corty {
    pub async fn new() -> Result<Self, Error> {
        // Create environment
        let environment = Self::initialize_environment().await;
        let agent = Self::initialize_agent().await?;

        // Register the agent
        let _ = environment.register_agent(agent, None).await?;
        Ok(Corty { environment })
    }

    async fn initialize_environment() -> Environment {
        // Create environment
        let environment = Environment::new(None).await;
        environment
    }

    pub async fn add_task(&self, task: String, agent_id: AgentID) -> Result<Uuid, Error> {
        Ok(self.environment.add_task(agent_id, task).await?)
    }

    async fn initialize_agent() -> Result<Arc<dyn RunnableAgent>, Error> {
        // Check if API key is set
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");

        // Initialize and configure the LLM client
        let llm: Arc<OpenAI> = LLMBuilder::<OpenAI>::new()
            .api_key(api_key)
            .model("gpt-4o")
            .max_tokens(2048)
            .temperature(0.1) // Lower temperature for more consistent code generation
            .stream(false)
            .build()
            .expect("Failed to build LLM");
        // Create memory with larger window for complex tasks
        let memory = Box::new(SlidingWindowMemory::new(30));

        // Create the coding agent
        let coding_agent = CodingAgent {};

        // Build the agent
        let agent = AgentBuilder::new(coding_agent)
            .with_llm(llm)
            .with_memory(memory)
            .build()?;

        Ok(agent)
    }

    pub async fn shutdown(&mut self) {
        // Shutdown the environment
        self.environment.shutdown().await;
    }
}
