import { test } from "node:test";
import assert from "node:assert/strict";
import { onRequest } from "../website/functions/api/invites/[code].js";
function database() {
  const rows = new Map();
  return {rows, prepare(sql) { return {bind(...args) {return {
    async first() { if(sql.includes("social_limits")) return {count:1}; const row=rows.get(args[0]); return row && row.expires_at > args[1] ? row : null; },
    async run() { if(sql.startsWith("INSERT OR IGNORE")) {const [key,kind,payload,expires_at]=args; if(rows.has(key)) return {meta:{changes:0}}; rows.set(key,{kind,payload,expires_at});} return {meta:{changes:1}}; }
  };}};}};
}
const request = (db,code,body) => onRequest({request:new Request(`https://luxmc-r92.pages.dev/api/invites/${code}`,body===undefined?{}:{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(body)}),params:{code},env:{SOCIAL_DB:db}});
test("short room and instance codes resolve without account and expire",async()=>{
  const db=database(); const codes=[];
  for(const [kind,payload] of [["room","LUX-1234|luxmc-world:fixture"],["instance",JSON.stringify({name:"Fixture",mcVersion:"1.21.4",loader:"vanilla",mods:[]})]]) {
    const response=await request(db,"create",{kind,payload}); assert.equal(response.status,200);
    const {code}=await response.json(); codes.push(code); assert.match(code,/^LUXMC1-[0-9]{16}$/);
    const resolved=await request(db,code); assert.equal(resolved.status,200);
    const value=await resolved.json(); assert.equal(value.kind,kind);assert.equal(value.payload,payload);
  }
  for(const row of db.rows.values()) row.expires_at=0;
  assert.equal((await request(db,codes[0])).status,404);
});
test("rejects malformed payloads and missing codes",async()=>{
 const db=database();
 for(const body of [{kind:"room",payload:"invalid"},{kind:"instance",payload:"null"},{kind:"instance",payload:"{}"},{kind:"other",payload:"x"}]) assert.equal((await request(db,"create",body)).status,400);
 assert.equal((await request(db,"../invalid")).status,400);
});
