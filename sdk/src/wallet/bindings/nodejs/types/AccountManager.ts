import type {
    Account,
    Auth,
    SyncOptions,
    ClientOptions,
    EventType,
    GenerateAddressOptions,
    LedgerNanoStatus,
    NodeInfoWrapper,
    WalletEvent,
    WalletApiEventHandler,
    CreateAccountPayload,
} from '.'

export interface AccountManager {
    id: string
    backup(destination: string, password: string): Promise<void>
    bech32ToHex(bech32Address: string): Promise<string>
    changeStrongholdPassword(currentPassword: string, newPassword: string): Promise<void>
    clearStrongholdPassword(): Promise<void>
    createAccount(payload: CreateAccountPayload): Promise<Account>
    destroy(): Promise<void>
    emitTestEvent(event: WalletEvent): Promise<void>
    generateAddress(
        accountIndex: number,
        internal: boolean,
        addressIndex: number,
        options?: GenerateAddressOptions,
        bech32Hrp?: string
    ): Promise<string>
    generateMnemonic(): Promise<string>
    getAccountIndexes(): Promise<number[]>
    getAccount(accountIndex: number): Promise<Account>
    getAccounts(): Promise<Account[]>
    getNodeInfo(url?: string, auth?: Auth): Promise<NodeInfoWrapper>
    getLedgerNanoStatus(): Promise<LedgerNanoStatus>
    hexToBech32(hex: string, bech32Hrp?: string): Promise<string>
    isStrongholdPasswordAvailable(): Promise<boolean>
    listen(eventTypes: EventType[], callback: WalletApiEventHandler): void
    clearListeners(eventTypes: EventType[]): Promise<void>
    removeLatestAccount(): Promise<void>
    recoverAccounts(
        accountStartIndex: number,
        accountGapLimit: number,
        addressGapLimit: number,
        syncOptions: SyncOptions,
    ): Promise<Account[]>
    restoreBackup(source: string, password: string, ignoreIfCoinTypeMismatch?: boolean): Promise<void>
    setClientOptions(options: ClientOptions): Promise<void>
    setStrongholdPassword(password: string): Promise<void>
    setStrongholdPasswordClearInterval(intervalInMilliseconds?: number): Promise<void>
    startBackgroundSync(options?: SyncOptions, intervalInMilliseconds?: number): Promise<void>
    stopBackgroundSync(): Promise<void>
    storeMnemonic(mnemonic: string): Promise<void>
    verifyMnemonic(mnemonic: string): Promise<void>
    updateNodeAuth(url: string, auth?: Auth): Promise<void>
}