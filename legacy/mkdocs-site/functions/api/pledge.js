// Cloudflare Pages Function：效忠计数「同源代理」
//
// 背景：前端原来跨域直连 *.workers.dev（pledge-counter.xuefeiyu2026.workers.dev），
// 该域名在国内手机网络下经常超时/被干扰，导致「效忠总人数」读不到、永远显示 0。
// 本 Function 把 /api/pledge 请求在 Cloudflare 内部网络转发给 Worker，
// 前端改为请求同源地址 /api/pledge——手机只要能打开网页，就一定能读到计数。
//
// 上游是 worker/pledge-counter.js（Cloudflare Worker，本地维护、手动部署）。
const UPSTREAM = "https://pledge-counter.xuefeiyu2026.workers.dev/api/pledge";

export async function onRequest(context) {
  const { request } = context;
  const method = request.method;

  // 把真实客户端 IP 传给 Worker：优先用 URL query 参数（100% 不会被 Cloudflare
  // 过滤）。不能只依赖 CF-Connecting-IP —— 经 Function 转发后会被 Cloudflare
  // 覆盖成 Pages 函数节点 IP（所有用户相同），导致防刷误伤所有人、计数被锁定。
  // 自定义头 X-Real-IP 也不可靠（可能被过滤），query 参数是最终保证。
  const clientIP = request.headers.get("CF-Connecting-IP") || "";
  const target = new URL(UPSTREAM);
  if (clientIP) target.searchParams.set("ip", clientIP);
  const headers = { "Content-Type": "application/json" };
  if (clientIP) headers["X-Real-IP"] = clientIP; // 双保险，保留自定义头

  // 原样透传 Worker 的响应（GET 200 {"count":N} / POST 200 或 429 {"error":"rate_limited"}）
  return fetch(target.toString(), { method, headers });
}
