export interface Statement {
    bind(...values: unknown[]): Statement;
    first<T = Record<string, any>>(): Promise<T | null>;
    all<T = Record<string, any>>(): Promise<{results:T[]}>;
    run(): Promise<{meta:{changes:number}}>;
}
export interface Database { prepare(query:string): Statement; batch(statements:Statement[]):Promise<{meta:{changes:number}}[]> }
export interface SiteEnvironment {
    SOCIAL_DB: Database;
    AUTH_PEPPER?: string;
    SOCIAL_STREAM_URL?: string;
    WINDOWS_LOCAL_VERSION?: string;
    OWNER_SOCIAL_ID?: string;
}
export interface SiteContext {
    request: Request;
    env: SiteEnvironment;
    params: Record<string,string>;
    waitUntil(promise:Promise<unknown>): void;
}
