from pathlib import Path
p=Path('tests/skin-import.browser.cjs')
s=p.read_text(encoding='utf-8', errors='replace').replace('N�o','Não').replace('poss�vel','possível').replace('indispon�vel','indisponível')
p.write_text(s,encoding='utf-8')
