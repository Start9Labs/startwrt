/**
 * Filter function type for transforming RPC call replies
 */
type FilterFn = (data: any, args: any[], ...extraArgs: any[]) => any;

/**
 * Invocation function returned by rpc.declare()
 */
type InvokeFn = (...params: any[]) => Promise<any>;

/**
 * Interceptor function for preprocessing RPC replies
 */
type InterceptorFn = (msg: any, req: object) => Promise<any> | any;

/**
 * Options for declaring RPC methods
 */
interface DeclareOptions {
    /** The name of the remote ubus object to invoke */
    object: string;

    /** The name of the remote ubus method to invoke */
    method: string;

    /** Lists the named parameters expected by the remote ubus RPC method */
    params?: string[];

    /** Describes the expected return data structure */
    expect?: Record<string, any>;

    /** Filter function to transform received reply data */
    filter?: FilterFn;

    /** If true, non-zero ubus call status codes are treated as fatal errors */
    reject?: boolean;

    /** Internal option for batch processing */
    nobatch?: boolean;
}

class Rpc {
    /**
     * Lists available remote ubus objects or the method signatures of specific objects
     */
    lis(...varargs: string[]): Promise<string[] | Record<string, Record<string, Record<string, string>>>>;

    /**
     * Describes a remote RPC call procedure and returns a function implementing it
     */
    declare(options: DeclareOptions, ...args: any[]): InvokeFn;

    /**
     * Returns the current RPC session id
     */
    getSessionID(): string;

    /**
     * Set the RPC session id to use
     */
    setSessionID(sid: string): void;

    /**
     * Returns the current RPC base URL
     */
    getBaseURL(): string;

    /**
     * Set the RPC base URL to use
     */
    setBaseURL(url: string): void;

    /**
     * Translates a numeric ubus error code into a human readable description
     */
    getStatusText(statusCode: number): string;

    /**
     * Registers a new interceptor function
     */
    addInterceptor(interceptorFn: InterceptorFn): InterceptorFn;

    /**
     * Removes a registered interceptor function
     */
    removeInterceptor(interceptorFn: InterceptorFn): boolean;

    private call(req: any, cb: Function, nobatch?: boolean): Promise<any[]>;
    private parseCallReply(req: any, res: any): void;
    private handleCallReply(req: any, msg: any): void;
}

export const rpc: Rpc;
