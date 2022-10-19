use std::collections::HashMap;

use bollard::{
    container::{ListContainersOptions, LogOutput, LogsOptions},
    Docker,
};
use futures::StreamExt;
use tokio::{sync::broadcast::Sender, task::JoinHandle};

use super::Flags;

pub(super) struct IncompatibleLedgerChecker {
    sender: Sender<Flags>,
    docker: Docker,
    tasks: Vec<JoinHandle<()>>,
}

impl IncompatibleLedgerChecker {
    pub fn new(sender: Sender<Flags>) -> Self {
        Self {
            sender,
            docker: Docker::connect_with_local_defaults().unwrap(),
            tasks: Vec::new(),
        }
    }

    pub async fn attach(&mut self) -> () {
        let containers = self
            .docker
            .list_containers(Some(ListContainersOptions {
                filters: HashMap::from([("name", vec!["validator"])]),
                ..Default::default()
            }))
            .await
            .unwrap();
        for container in containers {
            self.attach_to_container(container.id.unwrap()).await;
        }
    }

    async fn attach_to_container(&mut self, container_name: String) {
        let mut stream = self.docker.logs(
            container_name.as_str(),
            Some(LogsOptions::<&str> {
                stdout: true,
                follow: true,
                ..Default::default()
            }),
        );
        let sender = self.sender.clone();
        let task = tokio::spawn(async move {
            while let Some(Ok(log)) = stream.next().await {
                match log {
                    LogOutput::Console { message } => {
                        let string = std::str::from_utf8(&message).unwrap();
                        for line in string.lines() {
                            if line.contains("incompatible") {
                                sender
                                    .send(Flags::IncompatibleLedger(line.to_owned()))
                                    .unwrap();
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
        self.tasks.push(task);
    }
}

impl Drop for IncompatibleLedgerChecker {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort()
        }
    }
}
