const {spawn} = require('node:child_process');
const fs = require('node:fs');
const assert = require('node:assert/strict');

(async () => {
    const binary = process.argv[2];
    assert.ok(binary && fs.existsSync(binary));
    const java = process.argv[3] || 'C:/Users/preda/AppData/Roaming/github/Luxmc/data/java/21/bin/java.exe';
    const children = [];
    let javaOutput = '',nativeOutput = '';
    const execute = (command,args,onOutput) => {
        const child = spawn(command,args,{windowsHide:true,stdio:['ignore','pipe','pipe']});
        children.push(child);
        child.stdout.on('data',bytes => onOutput(bytes.toString()));
        child.stderr.on('data',bytes => onOutput(bytes.toString()));
        const finished = new Promise((resolve,reject) => {child.on('error',reject);child.on('exit',code => code === 0 ? resolve() : reject(new Error(`Process exited ${code}`)));});
        return {child,finished};
    };
    const timeout = setTimeout(() => children.forEach(child => child.kill()),45000);
    try {
        const reader = execute(java,['-cp','C:/Users/preda/AppData/Local/Luxmc-build/lan-java-probe','LanListProbe'],text => javaOutput+=text);
        const deadline = Date.now()+10000;
        while (!javaOutput.includes('READY') && Date.now()<deadline) await new Promise(resolve => setTimeout(resolve,30));
        assert.ok(javaOutput.includes('READY'),javaOutput);
        const native = execute(binary,['network::p2p_tunnel::tests::lan_discovery_on_physical_interface_connects_to_encrypted_proxy','--exact','--nocapture'],text => nativeOutput+=text);
        await Promise.all([reader.finished,native.finished]);
        assert.ok(javaOutput.includes('JAVA_LAN_DISCOVERY_AND_GAME_TRANSFER_OK'),javaOutput);
        assert.ok(nativeOutput.includes('test result: ok'),nativeOutput);
        fs.mkdirSync('docs/validation/v3.6/room-owner-cancellation',{recursive:true});
        fs.writeFileSync('docs/validation/v3.6/room-owner-cancellation/java-lan.json',JSON.stringify({passed:true,javaDefaultMulticastListener:true,gameTransferThroughDiscoveredAddress:true,javaOutput,nativeOutput},null,2));
        console.log('Java default multicast listener discovered the world and transferred game bytes through the advertised address.');
    } finally {clearTimeout(timeout);children.forEach(child => {if (child.exitCode == null) child.kill();});}
})().catch(error => {console.error(error);process.exitCode=1;});
