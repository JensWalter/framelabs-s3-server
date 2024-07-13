use aws_credential_types::{provider::ProvideCredentials, Credentials};

#[derive(Debug, Default)]
pub struct StaticCredentials {
    pub access_key: String,
    pub secret_key: String,
}

impl StaticCredentials {
    pub async fn load_credentials(&self) -> aws_credential_types::provider::Result {
        Ok(Credentials::new(
            self.access_key.clone(),
            self.secret_key.clone(),
            None,
            None,
            "StaticCredentials",
        ))
    }
}

impl ProvideCredentials for StaticCredentials {
    fn provide_credentials<'a>(
        &'a self,
    ) -> aws_credential_types::provider::future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        aws_credential_types::provider::future::ProvideCredentials::new(self.load_credentials())
    }
}
