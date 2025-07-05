import { Buffer } from "buffer";
import { Address } from '@stellar/stellar-sdk';
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from '@stellar/stellar-sdk/contract';
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Typepoint,
  Duration,
} from '@stellar/stellar-sdk/contract';
export * from '@stellar/stellar-sdk'
export * as contract from '@stellar/stellar-sdk/contract'
export * as rpc from '@stellar/stellar-sdk/rpc'

if (typeof window !== 'undefined') {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}


export const networks = {
  testnet: {
    networkPassphrase: "Test SDF Network ; September 2015",
    contractId: "CCTC7XNBQQHV7RJEJU5FSMY47L6KKTZHV4ZPKW2RPKMHS4S5JVBKM2R2",
  }
} as const


export interface Deposit {
  amount: i128;
  beneficiary: string;
  depositor: string;
  released: boolean;
  token: string;
}

export const Errors = {
  1: {message:"AlreadyInitialized"},
  2: {message:"NotInitialized"},
  3: {message:"Unauthorized"},
  4: {message:"InvalidAmount"},
  5: {message:"InvalidBeneficiary"},
  6: {message:"DepositExists"},
  7: {message:"DepositNotFound"},
  8: {message:"AlreadyReleased"}
}

export interface Client {
  /**
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * --- INICIALIZACIÓN ---
   * Inicializa el contrato estableciendo un propietario.
   * Solo se puede llamar una vez.
   */
  initialize: ({owner}: {owner: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a deposit transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * --- DEPÓSITO ---
   * Un usuario deposita tokens en el contrato para un beneficiario.
   */
  deposit: ({caller, beneficiary, token, amount, nonce}: {caller: string, beneficiary: string, token: string, amount: i128, nonce: Buffer}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a release_funds transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * --- LIBERACIÓN DE FONDOS ---
   * El propietario del contrato libera los fondos al beneficiario.
   */
  release_funds: ({deposit_id}: {deposit_id: Buffer}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a refund transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * --- REEMBOLSO ---
   * El depositante original solicita un reembolso.
   */
  refund: ({deposit_id}: {deposit_id: Buffer}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy(null, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAQAAAAAAAAAAAAAAB0RlcG9zaXQAAAAABQAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAtiZW5lZmljaWFyeQAAAAATAAAAAAAAAAlkZXBvc2l0b3IAAAAAAAATAAAAAAAAAAhyZWxlYXNlZAAAAAEAAAAAAAAABXRva2VuAAAAAAAAEw==",
        "AAAABAAAAAAAAAAAAAAABUVycm9yAAAAAAAACAAAAAAAAAASQWxyZWFkeUluaXRpYWxpemVkAAAAAAABAAAAAAAAAA5Ob3RJbml0aWFsaXplZAAAAAAAAgAAAAAAAAAMVW5hdXRob3JpemVkAAAAAwAAAAAAAAANSW52YWxpZEFtb3VudAAAAAAAAAQAAAAAAAAAEkludmFsaWRCZW5lZmljaWFyeQAAAAAABQAAAAAAAAANRGVwb3NpdEV4aXN0cwAAAAAAAAYAAAAAAAAAD0RlcG9zaXROb3RGb3VuZAAAAAAHAAAAAAAAAA9BbHJlYWR5UmVsZWFzZWQAAAAACA==",
        "AAAAAAAAAGotLS0gSU5JQ0lBTElaQUNJw5NOIC0tLQpJbmljaWFsaXphIGVsIGNvbnRyYXRvIGVzdGFibGVjaWVuZG8gdW4gcHJvcGlldGFyaW8uClNvbG8gc2UgcHVlZGUgbGxhbWFyIHVuYSB2ZXouAAAAAAAKaW5pdGlhbGl6ZQAAAAAAAQAAAAAAAAAFb3duZXIAAAAAAAATAAAAAA==",
        "AAAAAAAAAFEtLS0gREVQw5NTSVRPIC0tLQpVbiB1c3VhcmlvIGRlcG9zaXRhIHRva2VucyBlbiBlbCBjb250cmF0byBwYXJhIHVuIGJlbmVmaWNpYXJpby4AAAAAAAAHZGVwb3NpdAAAAAAFAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAAC2JlbmVmaWNpYXJ5AAAAABMAAAAAAAAABXRva2VuAAAAAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAVub25jZQAAAAAAA+4AAAAgAAAAAA==",
        "AAAAAAAAAFwtLS0gTElCRVJBQ0nDk04gREUgRk9ORE9TIC0tLQpFbCBwcm9waWV0YXJpbyBkZWwgY29udHJhdG8gbGliZXJhIGxvcyBmb25kb3MgYWwgYmVuZWZpY2lhcmlvLgAAAA1yZWxlYXNlX2Z1bmRzAAAAAAAAAQAAAAAAAAAKZGVwb3NpdF9pZAAAAAAD7gAAACAAAAAA",
        "AAAAAAAAAEAtLS0gUkVFTUJPTFNPIC0tLQpFbCBkZXBvc2l0YW50ZSBvcmlnaW5hbCBzb2xpY2l0YSB1biByZWVtYm9sc28uAAAABnJlZnVuZAAAAAAAAQAAAAAAAAAKZGVwb3NpdF9pZAAAAAAD7gAAACAAAAAA" ]),
      options
    )
  }
  public readonly fromJSON = {
    initialize: this.txFromJSON<null>,
        deposit: this.txFromJSON<null>,
        release_funds: this.txFromJSON<null>,
        refund: this.txFromJSON<null>
  }
}