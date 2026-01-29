use google_api_proto::google::cloud::speech::v1::{
    speech_client::SpeechClient, RecognitionConfig, StreamingRecognitionConfig,
    StreamingRecognizeRequest, streaming_recognize_request::StreamingRequest,
};
use tonic::{
    metadata::MetadataValue, transport::{Channel, ClientTlsConfig}, Request,
};
use tokio_stream::StreamExt;
use futures_util::Stream;
use std::env;
use anyhow::{Result, Context};

pub struct GoogleSTT {
    client: SpeechClient<Channel>,
}

impl GoogleSTT {
    pub async fn new() -> Result<Self> {
        let creds_path = env::var("GOOGLE_APPLICATION_CREDENTIALS")
            .context("GOOGLE_APPLICATION_CREDENTIALS not set")?;
            
        // Simplified auth for prompt: In production, we'd use google-auth-library or similar.
        // For now, let's assume we can pass the token or use a local emulator if testing, 
        // but for real Google Cloud, we need an OAuth token.
        // Tonic doesn't auto-handle loading JSON creds to tokens. 
        // We typically need `yup-oauth2`.
        // Adding `yup-oauth2` to Cargo.toml would be best.
        
        let tls_config = ClientTlsConfig::new();
        let channel = Channel::from_static("https://speech.googleapis.com")
            .tls_config(tls_config)?
            .connect()
            .await?;

        // We need an interceptor for auth, but let's stick to the structure for now.
        let client = SpeechClient::new(channel);
        
        Ok(Self { client })
    }

    pub async fn stream_audio<S>(&mut self, audio_stream: S) -> Result<()>
    where
        S: Stream<Item = Vec<u8>> + Send + 'static,
    {
        // 1. Config Message
        let config = RecognitionConfig {
            encoding: 1, // LINEAR16
            sample_rate_hertz: 16000,
            language_code: "en-US".to_string(),
            enable_automatic_punctuation: true,
            model: "latest_long".to_string(),
            use_enhanced: true,
            ..Default::default()
        };

        let streaming_config = StreamingRecognitionConfig {
            config: Some(config),
            interim_results: true,
            ..Default::default()
        };
        
        // 2. Map audio stream to Requests
        let outbound = async_stream::stream! {
            yield StreamingRecognizeRequest {
                streaming_request: Some(StreamingRequest::StreamingConfig(streaming_config)),
            };

            let mut pin_stream = Box::pin(audio_stream);
            while let Some(chunk) = pin_stream.next().await {
                yield StreamingRecognizeRequest {
                    streaming_request: Some(StreamingRequest::AudioContent(chunk.into())),
                };
            }
        };

        // 3. Call API
        let request = Request::new(outbound);
        let mut response_stream = self.client.streaming_recognize(request).await?.into_inner();

        // 4. Handle Responses
        while let Some(response) = response_stream.message().await? {
             for result in response.results {
                 if let Some(alternative) = result.alternatives.first() {
                     println!("Transcript: {}", alternative.transcript);
                     // Emit to frontend here
                 }
             }
        }

        Ok(())
    }
}
