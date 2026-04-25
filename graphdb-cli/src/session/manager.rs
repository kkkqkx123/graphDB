use crate::client::http::{GraphDBHttpClient, QueryResult};
use crate::session::variables::VariableStore;
use crate::utils::error::{CliError, Result};

#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: i64,
    pub username: String,
    pub current_space: Option<String>,
    pub host: String,
    pub port: u16,
    pub connected: bool,
    pub variable_store: VariableStore,
}

impl Session {
    pub fn new(session_id: i64, username: String, host: String, port: u16) -> Self {
        Self {
            session_id,
            username,
            current_space: None,
            host,
            port,
            connected: true,
            variable_store: VariableStore::new(),
        }
    }

    pub fn prompt(&self) -> String {
        if !self.connected {
            return "graphdb=# ".to_string();
        }

        let user_part = &self.username;
        let space_part = self.current_space.as_deref().unwrap_or("");

        if space_part.is_empty() {
            format!("graphdb({})=# ", user_part)
        } else {
            format!("graphdb({}:{})=# ", user_part, space_part)
        }
    }

    pub fn continuation_prompt(&self) -> String {
        if !self.connected {
            return "graphdb-# ".to_string();
        }

        let user_part = &self.username;
        let space_part = self.current_space.as_deref().unwrap_or("");

        if space_part.is_empty() {
            format!("graphdb({})-# ", user_part)
        } else {
            format!("graphdb({}:{})-# ", user_part, space_part)
        }
    }

    pub fn set_variable(&mut self, name: String, value: String) -> crate::utils::error::Result<()> {
        self.variable_store.set(name, value)
    }

    pub fn get_variable(&self, name: &str) -> Option<&String> {
        self.variable_store.get(name)
    }

    pub fn remove_variable(&mut self, name: &str) -> bool {
        self.variable_store.remove(name)
    }

    pub fn substitute_variables(&self, input: &str) -> crate::utils::error::Result<String> {
        self.variable_store.substitute(input)
    }

    pub fn conninfo(&self) -> String {
        let mut info = Vec::new();
        info.push(format!("Host: {}:{}", self.host, self.port));
        info.push(format!("Username: {}", self.username));
        info.push(format!(
            "Space: {}",
            self.current_space.as_deref().unwrap_or("(none)")
        ));
        info.push(format!("Session ID: {}", self.session_id));
        info.push(format!("Connected: {}", self.connected));
        info.join("\n")
    }

    #[deprecated(note = "Use variable_store directly")]
    pub fn variables(&self) -> &std::collections::HashMap<String, String> {
        self.variable_store.user_variables()
    }
}

pub struct SessionManager {
    client: GraphDBHttpClient,
    session: Option<Session>,
}

impl SessionManager {
    pub fn new(host: &str, port: u16) -> Self {
        let client = GraphDBHttpClient::new(host, port);
        Self {
            client,
            session: None,
        }
    }

    pub async fn connect(&mut self, username: &str, password: &str) -> Result<()> {
        let (session_id, _) = self.client.login(username, password).await?;

        let session = Session::new(
            session_id,
            username.to_string(),
            self.client
                .base_url()
                .trim_start_matches("http://")
                .trim_end_matches("/v1")
                .split(':')
                .next()
                .unwrap_or("127.0.0.1")
                .to_string(),
            self.client
                .base_url()
                .trim_start_matches("http://")
                .trim_end_matches("/v1")
                .split(':')
                .nth(1)
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
        );

        self.session = Some(session);
        Ok(())
    }

    pub async fn connect_with_host(
        &mut self,
        host: &str,
        port: u16,
        username: &str,
        password: &str,
    ) -> Result<()> {
        self.client = GraphDBHttpClient::new(host, port);
        self.connect(username, password).await
    }

    pub fn disconnect(&mut self) -> Result<()> {
        if self.session.is_none() {
            return Err(CliError::NotConnected);
        }
        self.session = None;
        Ok(())
    }

    pub async fn switch_space(&mut self, space: &str) -> Result<()> {
        let session = self.session.as_mut().ok_or(CliError::NotConnected)?;

        self.client.use_space(space).await?;
        session.current_space = Some(space.to_string());

        Ok(())
    }

    pub async fn execute_query(&self, query: &str) -> Result<QueryResult> {
        let session = self.session.as_ref().ok_or(CliError::NotConnected)?;

        let substituted = session.substitute_variables(query)?;
        self.client
            .execute_query(&substituted, session.session_id)
            .await
    }

    pub async fn execute_query_raw(&self, query: &str) -> Result<QueryResult> {
        let session = self.session.as_ref().ok_or(CliError::NotConnected)?;
        self.client.execute_query(query, session.session_id).await
    }

    pub async fn health_check(&self) -> Result<bool> {
        self.client.health_check().await
    }

    pub async fn list_spaces(&self) -> Result<Vec<crate::client::http::SpaceInfo>> {
        self.client.list_spaces().await
    }

    pub async fn list_tags(&self) -> Result<Vec<crate::client::http::TagInfo>> {
        let session = self.session.as_ref().ok_or(CliError::NotConnected)?;
        let space = session
            .current_space
            .as_deref()
            .ok_or(CliError::NoSpaceSelected)?;
        self.client.list_tags(space).await
    }

    pub async fn list_edge_types(&self) -> Result<Vec<crate::client::http::EdgeTypeInfo>> {
        let session = self.session.as_ref().ok_or(CliError::NotConnected)?;
        let space = session
            .current_space
            .as_deref()
            .ok_or(CliError::NoSpaceSelected)?;
        self.client.list_edge_types(space).await
    }

    pub fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    pub fn session_mut(&mut self) -> Option<&mut Session> {
        self.session.as_mut()
    }

    pub fn is_connected(&self) -> bool {
        self.session.is_some()
    }

    pub fn current_space(&self) -> Option<&str> {
        self.session
            .as_ref()
            .and_then(|s| s.current_space.as_deref())
    }

    pub fn client(&self) -> &GraphDBHttpClient {
        &self.client
    }
}
