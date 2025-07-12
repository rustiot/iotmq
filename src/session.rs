use crate::Context;

pub struct Session {}

impl Session {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(self) {
        println!("session running...");
    }

    async fn run_loop() {}
}
