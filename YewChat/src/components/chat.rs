use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{User, services::websocket::WebsocketService};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    _producer: Box<dyn Bridge<EventBus>>,
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        
        // Get current user context
        let (current_user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let current_username = current_user.username.borrow().clone();

        html! {
            <div class="flex w-screen h-screen bg-gray-50">
                // Sidebar dengan users
                <div class="flex-none w-80 h-full bg-white border-r border-gray-200 shadow-sm">
                    <div class="p-6 border-b border-gray-100">
                        <h2 class="text-2xl font-bold text-gray-900">{"Online Users"}</h2>
                        <p class="text-sm text-gray-500 mt-1">{format!("{} users online", self.users.len())}</p>
                    </div>
                    <div class="overflow-y-auto h-full pb-32">
                        {
                            self.users.iter().map(|u| {
                                let is_current_user = u.name == current_username;
                                let user_class = if is_current_user {
                                    "flex items-center p-4 mx-3 my-2 bg-gradient-to-r from-blue-500 to-purple-600 text-white rounded-xl shadow-lg transform hover:scale-105 transition-all duration-200"
                                } else {
                                    "flex items-center p-4 mx-3 my-2 bg-white hover:bg-gray-50 rounded-xl shadow-sm border border-gray-100 hover:shadow-md transition-all duration-200"
                                };
                                
                                html!{
                                    <div class={user_class}>
                                        <div class="relative">
                                            <img class="w-14 h-14 rounded-full ring-2 ring-white shadow-md" src={u.avatar.clone()} alt="avatar"/>
                                            <div class="absolute -bottom-1 -right-1 w-5 h-5 bg-green-400 rounded-full border-2 border-white"></div>
                                        </div>
                                        <div class="ml-4 flex-grow">
                                            <div class="flex items-center justify-between">
                                                <h3 class={if is_current_user { "font-semibold text-white" } else { "font-semibold text-gray-900" }}>
                                                    {u.name.clone()}
                                                    if is_current_user {
                                                        <span class="ml-2 text-xs bg-white bg-opacity-20 px-2 py-1 rounded-full">{"You"}</span>
                                                    }
                                                </h3>
                                            </div>
                                            <p class={if is_current_user { "text-white text-opacity-90 text-sm" } else { "text-gray-500 text-sm" }}>
                                                {"Active now"}
                                            </p>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                </div>

                // Main chat area
                <div class="flex-1 flex flex-col bg-white">
                    // Header
                    <div class="flex items-center justify-between p-6 border-b border-gray-200 bg-white">
                        <div class="flex items-center">
                            <div class="w-12 h-12 bg-gradient-to-r from-blue-500 to-purple-600 rounded-full flex items-center justify-center text-white font-bold text-xl shadow-lg">
                                {"💬"}
                            </div>
                            <div class="ml-4">
                                <h1 class="text-2xl font-bold text-gray-900">{"Group Chat"}</h1>
                                <p class="text-sm text-gray-500">{"Stay connected with everyone"}</p>
                            </div>
                        </div>
                        <div class="flex items-center space-x-2">
                            <div class="w-3 h-3 bg-green-400 rounded-full animate-pulse"></div>
                            <span class="text-sm text-gray-500">{"Connected"}</span>
                        </div>
                    </div>

                    // Messages area
                    <div class="flex-1 overflow-y-auto p-6 space-y-4 bg-gray-50">
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from);
                                let is_own_message = m.from == current_username;
                                
                                if let Some(user) = user {
                                    if is_own_message {
                                        // Own message - right aligned
                                        html!{
                                            <div class="flex justify-end">
                                                <div class="flex items-end space-x-3 max-w-md">
                                                    <div class="bg-gradient-to-r from-blue-500 to-purple-600 text-white p-4 rounded-t-2xl rounded-bl-2xl shadow-lg">
                                                        <div class="font-medium text-sm mb-1">{"You"}</div>
                                                        <div class="text-sm">
                                                            if m.message.ends_with(".gif") {
                                                                <img class="rounded-lg max-w-full" src={m.message.clone()}/>
                                                            } else {
                                                                {m.message.clone()}
                                                            }
                                                        </div>
                                                    </div>
                                                    <img class="w-10 h-10 rounded-full ring-2 ring-blue-200" src={user.avatar.clone()} alt="avatar"/>
                                                </div>
                                            </div>
                                        }
                                    } else {
                                        // Other's message - left aligned
                                        html!{
                                            <div class="flex justify-start">
                                                <div class="flex items-end space-x-3 max-w-md">
                                                    <img class="w-10 h-10 rounded-full ring-2 ring-gray-200" src={user.avatar.clone()} alt="avatar"/>
                                                    <div class="bg-white p-4 rounded-t-2xl rounded-br-2xl shadow-md border border-gray-100">
                                                        <div class="font-medium text-sm mb-1 text-gray-900">{m.from.clone()}</div>
                                                        <div class="text-sm text-gray-700">
                                                            if m.message.ends_with(".gif") {
                                                                <img class="rounded-lg max-w-full" src={m.message.clone()}/>
                                                            } else {
                                                                {m.message.clone()}
                                                            }
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    }
                                } else {
                                    html!{}
                                }
                            }).collect::<Html>()
                        }
                    </div>

                    // Input area
                    <div class="p-6 bg-white border-t border-gray-200">
                        <div class="flex items-center space-x-4">
                            <div class="flex-1 relative">
                                <input 
                                    ref={self.chat_input.clone()} 
                                    type="text" 
                                    placeholder="Type your message..." 
                                    class="w-full py-4 px-6 bg-gray-100 rounded-full outline-none focus:ring-2 focus:ring-blue-500 focus:bg-white border border-transparent focus:border-blue-200 transition-all duration-200 text-gray-900 placeholder-gray-500" 
                                    name="message" 
                                    required=true 
                                />
                            </div>
                            <button 
                                onclick={submit} 
                                class="w-14 h-14 bg-gradient-to-r from-blue-500 to-purple-600 hover:from-blue-600 hover:to-purple-700 rounded-full flex items-center justify-center text-white shadow-lg hover:shadow-xl transform hover:scale-105 transition-all duration-200"
                            >
                                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" class="transform rotate-45">
                                    <path d="M7 17L17 7M17 7H7M17 7V17" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                                </svg>
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}