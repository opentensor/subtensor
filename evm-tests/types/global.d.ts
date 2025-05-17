// Declaration file for modules with missing type declarations

declare module '@polkadot-api/descriptors' {
  export const devnet: any;
  export interface MultiAddress {
    Id: (address: string) => any;
  }
}

declare module 'polkadot-api' {
  export interface TypedApi<T> {
    query: {
      [module: string]: {
        [method: string]: {
          getValue: (address?: string) => Promise<any>;
          watchValue: (address?: string) => {
            subscribe: (callbacks: {
              next: (value: any) => void;
              error: (err: any) => void;
              complete: () => void;
            }) => { unsubscribe: () => void };
          };
        };
      };
    };
    tx: {
      [module: string]: {
        [method: string]: (...args: any[]) => any;
      };
    };
    txFromCallData: (raw: any) => Promise<any>;
    [key: string]: any;
  }

  export interface PolkadotSigner {
    publicKey: Uint8Array;
    [key: string]: any;
  }

  export interface Transaction<A, B, C, D> {
    signSubmitAndWatch(signer: PolkadotSigner): {
      subscribe: (callbacks: {
        next: (value: any) => void;
        error: (err: any) => void;
        complete: () => void;
      }) => { unsubscribe: () => void };
    };
    [key: string]: any;
  }

  export function createClient(provider: any): any;
  export type Binary = any;
}

declare module 'polkadot-api/ws-provider/web' {
  export function getWsProvider(url: string): any;
}

declare module 'polkadot-api/signer' {
  export function getPolkadotSigner(publicKey: Uint8Array, type: string, sign: any): any;
}

// Mocha globals
declare function describe(name: string, fn: () => void): void;
declare function it(name: string, fn: () => void | Promise<void>): void;
declare function before(fn: () => void | Promise<void>): void;
declare function after(fn: () => void | Promise<void>): void;
declare function beforeEach(fn: () => void | Promise<void>): void;
declare function afterEach(fn: () => void | Promise<void>): void;
