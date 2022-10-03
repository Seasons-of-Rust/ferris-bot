use crate::{api, Error};
use indexmap::IndexMap;
use reqwest::Client as HttpClient;
use serenity::{model::channel::Message, prelude::Context};
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};
use tracing::{error, info};

pub const PREFIX: &str = "?";

type ResultFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + Send>>;

pub trait AsyncFn<T>: 'static {
    fn call(&self, args: Arc<Args>) -> ResultFuture<T, Error>;
}

impl<F, G, T> AsyncFn<T> for F
where
    F: Fn(Arc<Args>) -> G + 'static,
    G: Future<Output = Result<T, Error>> + Send + 'static,
{
    fn call(&self, args: Arc<Args>) -> ResultFuture<T, Error> {
        let fut = (self)(args);
        Box::pin(async move { fut.await })
    }
}

pub type Handler = dyn AsyncFn<()> + Send + Sync;
pub type Auth = dyn AsyncFn<bool> + Send + Sync;

pub enum CommandKind {
    Base,
    Protected,
    Help,
}

pub struct Command {
    pub kind: CommandKind,
    pub auth: &'static Auth,
    pub handler: &'static Handler,
}

impl Command {
    pub fn new(handler: &'static Handler) -> Self {
        Self {
            kind: CommandKind::Base,
            auth: &|_| async { Ok(true) },
            handler,
        }
    }

    pub fn new_with_auth(handler: &'static Handler, auth: &'static Auth) -> Self {
        Self {
            kind: CommandKind::Protected,
            auth,
            handler,
        }
    }

    pub fn help() -> Self {
        Self {
            kind: CommandKind::Help,
            auth: &|_| async { Ok(true) },
            handler: &|_| async { Ok(()) },
        }
    }
}

pub struct Args {
    pub cx: Context,
    pub msg: Message,
    pub params: HashMap<&'static str, String>,
    pub http: Arc<HttpClient>,
}

pub struct Params {
    pub warn: Option<String>,
    pub channel: Option<String>,
    pub mode: Option<String>,
    pub edition: Option<String>,
}

impl Params {
    pub fn default() -> Self {
        Self {
            warn: None,
            channel: None,
            mode: None,
            edition: None,
        }
    }
}

async fn execute_command(args: Arc<Args>, handler: &'static Handler) {
    info!("Executing command");
    if let Err(e) = handler.call(args).await {
        error!("{}", e);
    }
}

// pub struct Commands {
//     state_machine: StateMachine,
//     command_map: HashMap<usize, Arc<Command>>,
//     menu: Option<IndexMap<&'static str, (&'static str, &'static Auth)>>,
// }

// impl Commands {
//     pub fn new() -> Self {
//         Self {
//             state_machine: StateMachine::new(),
//             command_map: HashMap::new(),
//             menu: Some(IndexMap::new()),
//         }
//     }

//     pub fn add(&mut self, input: &'static str, command: Command) {
//         info!("Adding command {}", &input);
//         let mut state = 0;

//         let mut reused_space_state = None;
//         let mut opt_final_states = vec![];

//         let handler = Arc::new(command);

//         if reused_space_state.is_some() {
//             opt_final_states.iter().for_each(|state| {
//                 self.state_machine.set_final_state(*state);
//                 self.command_map.insert(*state, handler.clone());
//             });
//         } else {
//             self.state_machine.set_final_state(state);
//             self.command_map.insert(state, handler.clone());
//         }
//     }


//     pub async fn execute(&self, cx: Context, msg: Message, http: Arc<HttpClient>) {
//         let message = &msg.content;
//         if !msg.is_own(&cx) && message.starts_with(PREFIX) {
//             if let Some(matched) = self.state_machine.process(message) {
//                 info!("Processing command: {}", message);
//                 let args = Arc::new(Args {
//                     cx,
//                     msg,
//                     params: matched.params,
//                     http: http.clone(),
//                 });

//                 let command = self.command_map.get(&matched.state).unwrap();

//                 match command.kind {
//                     CommandKind::Base => {
//                         execute_command(args.clone(), command.handler).await;
//                     }
//                     CommandKind::Protected => match command.auth.call(args.clone()).await {
//                         Ok(true) => {
//                             execute_command(args.clone(), command.handler).await;
//                         }
//                         Ok(false) => {
//                             info!("Not executing command, unauthorized");
//                             if let Err(e) = api::send_reply(
//                                 args.clone(),
//                                 "You do not have permission to run this command",
//                             )
//                             .await
//                             {
//                                 error!("{}", e);
//                             }
//                         }
//                         Err(e) => error!("{}", e),
//                     },
//                     CommandKind::Help => {
//                         let output =
//                             api::main_menu(args.clone(), self.menu.as_ref().unwrap()).await;
//                         if let Err(e) =
//                             api::send_reply(args.clone(), &format!("```{}```", &output)).await
//                         {
//                             error!("{}", e)
//                         }
//                     }
//                 };
//             }
//         }
//     }
// }

// fn key_value_pair(s: &'static str) -> Option<&'static str> {
//     s.match_indices("={}")
//         .next()
//         .map(|pair| {
//             let name = &s[0..pair.0];
//             if name.len() > 0 {
//                 Some(name)
//             } else {
//                 None
//             }
//         })
//         .flatten()
// }
