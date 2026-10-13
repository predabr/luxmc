export async function onRequest(context: {request:Request;next():Promise<Response>}) {
    const response=await context.next();
    const path=new URL(context.request.url).pathname;
    if (!/^\/(?:api|download)\//.test(path)) return response;
    const secured=new Response(response.body,response);
    secured.headers.set('X-Content-Type-Options','nosniff');
    secured.headers.set('Referrer-Policy','strict-origin-when-cross-origin');
    secured.headers.set('Strict-Transport-Security','max-age=31536000');
    if (context.request.method==='GET' && path==='/api/player' && response.status===200 && response.headers.get('Content-Type')?.startsWith('text/html')) {
        secured.headers.delete('X-Frame-Options');
    } else {
        secured.headers.set('X-Frame-Options','DENY');
        secured.headers.set('Content-Security-Policy',"default-src 'none'; frame-ancestors 'none'");
    }
    return secured;
}
