from pathlib import Path
p=Path('tests/appearance-microsoft.browser.cjs')
s=p.read_text()
s=s.replace("if(command==='auth_change_skin')", "if(command==='plugin:dialog|open')return 'C:/fixture/cape.png';\n   if(command==='auth_read_local_texture')return window.fixtureCape;\n   if(command==='auth_change_skin')")
s=s.replace("{name:'Adicionar capa PNG',exact:true}", "{name:'Importar capa PNG',exact:true}")
s=s.replace("await page.getByRole('button',{name:'Capa Luxmc Oficial',exact:true}).click();", "await page.evaluate(async()=>{const {getFullCapeDataUrl}=await import('/src/lib/utils/capeTextures.ts');window.fixtureCape=getFullCapeDataUrl('luxmc');});\n await cape.click();\n await page.getByText('Sua capa atual',{exact:true}).waitFor();")
s=s.replace("assert.equal(await page.getByRole('button',{name:'Capa Luxmc Oficial',exact:true}).getAttribute('aria-pressed'),'true');", "await page.getByText('Sua capa atual',{exact:true}).waitFor();\n assert.equal(await page.getByRole('button',{name:'Sem capa',exact:true}).getAttribute('aria-pressed'),'false');\n assert.equal(await page.getByRole('button',{name:'Capa Luxmc Oficial',exact:true}).count(),0);")
p.write_text(s)
p=Path('tests/skin-import.browser.cjs')
s=p.read_text()
a=s.index("    const capes = ['Luxmc Oficial'")
b=s.index("    const canvas = page.getByRole('region'",a)
s=s[:a]+'''    assert.equal(await page.getByRole('button', { name: 'Capa Luxmc Oficial', exact: true }).count(), 0);
    await page.evaluate(async () => {
        const { getFullCapeDataUrl } = await import('/src/lib/utils/capeTextures.ts');
        window.importPath = 'C:/fixture/cape.png';
        window.importTexture = getFullCapeDataUrl('luxmc');
    });
    const importCape = page.getByRole('button', { name: 'Importar capa PNG', exact: true });
    await importCape.click();
    await page.getByText('Sua capa atual', {exact:true}).waitFor();
    await page.waitForFunction(() => window.importViewer?.playerObject.cape.visible);
    await page.evaluate(() => {
        const viewer = window.importViewer;
        const load = viewer.loadCape;
        viewer.loadCape = function(...args) { this.loadCape = load; throw new Error('Capa indisponível'); };
        window.importTexture = document.createElement('canvas').toDataURL();
    });
    await page.getByRole('button', { name: 'Sem capa', exact: true }).click();
    await page.getByRole('button', { name: 'Personalizada', exact: true }).click();
    await page.getByText('Não foi possível carregar a capa. Selecione outra ou importe o PNG novamente.').waitFor();
    assert.equal(await page.evaluate(() => window.importViewer.playerObject.cape.visible), false);
    await page.getByRole('button', { name: 'Sem capa', exact: true }).click();
    await page.getByRole('button', { name: 'Personalizada', exact: true }).click();
    await page.waitForFunction(() => window.importViewer.playerObject.cape.visible);
    assert.equal(await page.getByText('Não foi possível carregar a capa. Selecione outra ou importe o PNG novamente.').count(), 0);
    await page.getByRole('button', { name: 'Sem capa', exact: true }).click();
    await page.waitForFunction(() => !window.importViewer.playerObject.cape.visible);
''' +s[b:]
s=s.replace("{ name: 'Capa Nenhuma', exact: true }", "{ name: 'Sem capa', exact: true }")
s=s.replace('builtinCapes: capes.length', 'builtinCapeCatalogRemoved: true, importedCapePreserved: true')
p.write_text(s)
