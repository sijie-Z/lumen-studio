import { request } from "./api";

export interface ChatMessage {
  role: "user" | "assistant";
  content: string;
}

export interface ChatResponse {
  reply: string;
  mode: "local" | "llm";
  sources: string[];
}

export async function sendChat(messages: ChatMessage[]) {
  return request<ChatResponse>("/ai/chat", {
    method: "POST",
    body: JSON.stringify({ messages })
  });
}
