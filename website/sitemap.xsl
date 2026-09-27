<?xml version="1.0" encoding="UTF-8"?>
<xsl:stylesheet version="2.0" 
                xmlns:html="http://www.w3.org/TR/REC-html40"
                xmlns:sitemap="http://www.sitemaps.org/schemas/sitemap/0.9"
                xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:output method="html" version="1.0" encoding="UTF-8" indent="yes"/>
  <xsl:template match="/">
    <html xmlns="http://www.w3.org/1999/xhtml" lang="pt-BR">
      <head>
        <title>Sitemap XML · Luxmc</title>
        <meta http-equiv="Content-Type" content="text/html; charset=utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <style type="text/css">
          body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "JetBrains Mono", monospace, sans-serif;
            background-color: #000000;
            color: #d4d4d8;
            margin: 0;
            padding: 40px 20px;
          }
          .container {
            max-width: 900px;
            margin: 0 auto;
            background: #09090b;
            border: 1px solid rgba(255, 255, 255, 0.12);
            border-radius: 8px;
            padding: 32px;
            box-sizing: border-box;
          }
          h1 {
            color: #ffffff;
            font-size: 24px;
            font-weight: 800;
            margin: 0 0 8px 0;
            letter-spacing: -0.03em;
          }
          .tag {
            font-family: monospace;
            font-size: 11px;
            color: #10b981;
            text-transform: uppercase;
            letter-spacing: 0.08em;
            margin-bottom: 12px;
            display: inline-block;
          }
          p {
            font-size: 13px;
            color: #8e8e93;
            margin-bottom: 24px;
            line-height: 1.5;
          }
          .table-wrap {
            overflow-x: auto;
          }
          table {
            width: 100%;
            border-collapse: collapse;
            font-size: 12px;
            font-family: monospace;
          }
          th {
            background-color: #111114;
            color: #a1a1aa;
            text-align: left;
            padding: 12px 14px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.12);
            text-transform: uppercase;
            font-size: 10.5px;
            letter-spacing: 0.05em;
          }
          td {
            padding: 12px 14px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.06);
          }
          tr:hover td {
            background-color: rgba(255, 255, 255, 0.03);
          }
          a {
            color: #ffffff;
            text-decoration: none;
          }
          a:hover {
            color: #10b981;
            text-decoration: underline;
          }
          .footer {
            margin-top: 24px;
            font-size: 11px;
            color: #52525b;
            font-family: monospace;
            text-transform: uppercase;
          }
        </style>
      </head>
      <body>
        <div class="container">
          <span class="tag">// SITEMAP OFICIAL LUXMC // INDEXAÇÃO GOOGLE &amp; EDGE</span>
          <h1>Mapa do Site (XML)</h1>
          <p>Este documento é lido automaticamente por rastreadores de busca como Googlebot e Bingbot. Todas as páginas abaixo são canônicas, seguras e ativas no ecossistema Luxmc.</p>
          <div class="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>URL Canônica</th>
                  <th>Prioridade</th>
                  <th>Frequência</th>
                  <th>Última Atualização</th>
                </tr>
              </thead>
              <tbody>
                <xsl:for-each select="sitemap:urlset/sitemap:url">
                  <tr>
                    <td>
                      <xsl:variable name="itemURL">
                        <xsl:value-of select="sitemap:loc"/>
                      </xsl:variable>
                      <a href="{$itemURL}">
                        <xsl:value-of select="sitemap:loc"/>
                      </a>
                    </td>
                    <td style="color: #10b981; font-weight: bold;"><xsl:value-of select="sitemap:priority"/></td>
                    <td><xsl:value-of select="sitemap:changefreq"/></td>
                    <td><xsl:value-of select="sitemap:lastmod"/></td>
                  </tr>
                </xsl:for-each>
              </tbody>
            </table>
          </div>
          <div class="footer">
            Total de URLs: <xsl:value-of select="count(sitemap:urlset/sitemap:url)"/> · Cloudflare Pages Edge
          </div>
        </div>
      </body>
    </html>
  </xsl:template>
</xsl:stylesheet>
