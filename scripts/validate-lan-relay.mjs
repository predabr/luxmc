import { spawn, execFileSync } from 'node:child_process';
import { randomBytes } from 'node:crypto';
import { join } from 'node:path';
import { writeFileSync } from 'node:fs';
import net from 'node:net';

const directory=join(process.env.TEMP,'luxmc-lan-loader-validation-v2.6.4');
const environment={...process.env,ET_NETWORK_NAME:`luxmc-validation-${randomBytes(16).toString('hex')}`,ET_NETWORK_SECRET:randomBytes(32).toString('hex')};
const children=[];
const ports=[];
const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
async function port() {const server=net.createServer();await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const value=server.address().port;await new Promise(resolve=>server.close(resolve));return value;}
function start(index){
    const child=spawn(join(directory,'easytier-core.exe'),['--no-tun','--ipv4',`10.100.1.${index+1}/24`,'--hostname',`Luxmc-validation-${index}`,'--peers','tcp://dreamlife.indevs.in:11010','--no-listener','--rpc-portal',`127.0.0.1:${ports[index]}`,'--rpc-portal-whitelist','127.0.0.1','--relay-network-whitelist','--console-log-level','off'],{cwd:directory,env:environment,windowsHide:true,stdio:'ignore'});
    children[index]=child;return child;
}
function peers(index){try{return JSON.parse(execFileSync(join(directory,'easytier-cli.exe'),['--rpc-portal',`127.0.0.1:${ports[index]}`,'--output','json','peer'],{cwd:directory,windowsHide:true,encoding:'utf8',timeout:6000}));}catch{return [];}}
function connected(){return peers(0).some(row=>row.hostname==='Luxmc-validation-1')&&peers(1).some(row=>row.hostname==='Luxmc-validation-0');}
const receipt={adapterCreated:false,independentProcesses:2,connected:false,reconnected:false,samples:0,missedSamples:0};
try{
    ports.push(await port(),await port());start(0);start(1);
    for(let attempt=0;attempt<30&&!receipt.connected;attempt++){await sleep(2000);receipt.connected=connected();}
    if(!receipt.connected)throw new Error('Os dois nós não se encontraram no relay configurado.');
    console.log('Dois nós independentes conectados pelo relay; observando estabilidade.');
    for(let attempt=0;attempt<12;attempt++){await sleep(5000);receipt.samples++;if(!connected())receipt.missedSamples++;}
    children[1].kill();await sleep(1000);start(1);
    for(let attempt=0;attempt<30&&!receipt.reconnected;attempt++){await sleep(2000);receipt.reconnected=connected();}
    if(!receipt.reconnected)throw new Error('O nó reiniciado não reconectou.');
    console.log('Reconexão após reiniciar um dos nós confirmada.');
    for(let attempt=0;attempt<12;attempt++){await sleep(5000);receipt.samples++;if(!connected())receipt.missedSamples++;}
    if(receipt.missedSamples)throw new Error('Foram observadas falhas de presença durante a amostragem.');
    console.log(JSON.stringify(receipt));
}catch(error){receipt.error=String(error);console.error(receipt.error);process.exitCode=1;}
finally{children.forEach(child=>child.kill());writeFileSync('docs/validation/v3.6/lan-relay-results.json',JSON.stringify(receipt,null,2)+'\n');}
