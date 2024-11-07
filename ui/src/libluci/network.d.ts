interface WifiEncryption {
    enabled: boolean;
    wep?: string[];
    wpa?: number[];
    authentication?: string[];
    ciphers?: string[];
}

interface WifiRateEntry {
    drop_misc?: number;
    packets: number;
    bytes: number;
    failed?: number;
    retries?: number;
    is_ht: boolean;
    is_vht: boolean;
    mhz: number;
    rate: number;
    mcs?: number;
    "40mhz"?: number;
    short_gi?: boolean;
    nss?: number;
    he?: boolean;
    he_gi?: number;
    he_dcm?: number;
}

interface WifiPeerEntry {
    mac: string;
    signal: number;
    signal_avg?: number;
    noise?: number;
    inactive: number;
    connected_time: number;
    thr?: number;
    authorized: boolean;
    authenticated: boolean;
    preamble: string;
    wme: boolean;
    mfp: boolean;
    tdls: boolean;
    "mesh llid"?: number;
    "mesh plid"?: number;
    "mesh plink"?: string;
    "mesh local PS"?: number;
    "mesh peer PS"?: number;
    "mesh non-peer PS"?: number;
    rx: WifiRateEntry;
    tx: WifiRateEntry;
}

interface WifiScanResult {
    ssid: string;
    bssid: string;
    mode: string;
    channel: number;
    signal: number;
    quality: number;
    quality_max: number;
    encryption: WifiEncryption;
}

interface SwitchTopology {
    netdevs: Record<number, string>;
    ports: Array<{
        num: number;
        label: string;
        device?: string;
        tagged?: boolean;
    }>;
}

class Protocol {
    constructor(name: string);
    get(opt: string): string | string[] | null;
    set(opt: string, val: string | string[] | null): void;
    getIfname(): string | null;
    getProtocol(): string | null;
    getI18n(): string;
    getType(): string | null;
    getName(): string;
    getUptime(): number;
    getExpiry(): number;
    getMetric(): number;
    getZoneName(): string | null;
    getIPAddr(): string | null;
    getIPAddrs(): string[];
    getNetmask(): string | null;
    getGatewayAddr(): string | null;
    getDNSAddrs(): string[];
    getIP6Addr(): string | null;
    getIP6Addrs(): string[];
    getGateway6Addr(): string | null;
    getDNS6Addrs(): string[];
    getIP6Prefix(): string | null;
    getErrors(): string[] | null;
    isBridge(): boolean;
    getOpkgPackage(): string | null;
    isCreateable(ifname: string): Promise<void>;
    isInstalled(): boolean;
    isVirtual(): boolean;
    isFloating(): boolean;
    isDynamic(): boolean;
    isAlias(): string | null;
    isEmpty(): boolean;
    isUp(): boolean;
    addDevice(device: Protocol | Device | WifiDevice | WifiNetwork | string): boolean;
    deleteDevice(device: Protocol | Device | WifiDevice | WifiNetwork | string): boolean;
    getDevice(): Device;
    getL2Device(): Device | null;
    getL3Device(): Device | null;
    getDevices(): Device[] | null;
    containsDevice(device: Protocol | Device | WifiDevice | WifiNetwork | string): boolean;
    deleteConfiguration(): void | Promise<any>;
}

class Device {
    constructor(device: string, network?: Protocol);
    getName(): string;
    getMAC(): string | null;
    getMTU(): number;
    getIPAddrs(): string[];
    getIP6Addrs(): string[];
    getType(): string;
    getShortName(): string;
    getI18n(): string;
    getTypeI18n(): string;
    getPorts(): Device[] | null;
    getBridgeID(): string | null;
    getBridgeSTP(): boolean;
    isUp(): boolean;
    isBridge(): boolean;
    isBridgePort(): boolean;
    getTXBytes(): number;
    getRXBytes(): number;
    getTXPackets(): number;
    getRXPackets(): number;
    getCarrier(): boolean;
    getSpeed(): number | null;
    getDuplex(): string | null;
    getNetwork(): Protocol | null;
    getNetworks(): Protocol[];
    getWifiNetwork(): WifiNetwork | null;
    getParent(): Device | null;
}

class WifiDevice {
    constructor(name: string, radiostate: any);
    get(opt: string): string | string[] | null;
    set(opt: string, value: string | string[] | null): void;
    isDisabled(): boolean;
    getName(): string;
    getHWModes(): string[];
    getHTModes(): string[] | null;
    getI18n(): string;
    getScanList(): Promise<WifiScanResult[]>;
    isUp(): boolean;
    getWifiNetwork(network: string): Promise<WifiNetwork>;
    getWifiNetworks(): Promise<WifiNetwork[]>;
    addWifiNetwork(options?: Record<string, string | string[]>): Promise<WifiNetwork | null>;
    deleteWifiNetwork(network: string | WifiNetwork): Promise<boolean>;
}

class WifiNetwork {
    constructor(sid: string, radioname: string, radiostate: any, netid: string, netstate: any, hostapd: any);
    get(opt: string): string | string[] | null;
    set(opt: string, value: string | string[] | null): void;
    isDisabled(): boolean;
    getMode(): string;
    getSSID(): string | null;
    getMeshID(): string | null;
    getBSSID(): string | null;
    getNetworkNames(): string[];
    getID(): string;
    getName(): string;
    getIfname(): string | null;
    getVlanIfnames(): string[];
    getWifiDeviceName(): string | null;
    getWifiDevice(): Promise<WifiDevice>;
    isUp(): boolean;
    getActiveMode(): string;
    getActiveModeI18n(): string;
    getActiveSSID(): string;
    getActiveBSSID(): string;
    getActiveEncryption(): string;
    getAssocList(): Promise<WifiPeerEntry[]>;
    getFrequency(): string | null;
    getBitRate(): number | null;
    getChannel(): number | null;
    getSignal(): number;
    getNoise(): number;
    getCountryCode(): string;
    getTXPower(): number | null;
    getTXPowerOffset(): number;
    getSignalLevel(signal?: number, noise?: number): number;
    getSignalPercent(): number;
    getShortName(): string;
    getI18n(): string;
    getNetwork(): Protocol | null;
    getNetworks(): Protocol[];
    getDevice(): Device;
    isClientDisconnectSupported(): boolean;
    disconnectClient(mac: string, deauth?: boolean, reason?: number, ban_time?: number): Promise<number>;
}

class Hosts {
    constructor(hosts: any);
    getHostnameByMACAddr(mac: string): string | null;
    getIPAddrByMACAddr(mac: string): string | null;
    getIP6AddrByMACAddr(mac: string): string | null;
    getHostnameByIPAddr(ipaddr: string): string | null;
    getMACAddrByIPAddr(ipaddr: string): string | null;
    getHostnameByIP6Addr(ip6addr: string): string | null;
    getMACAddrByIP6Addr(ip6addr: string): string | null;
    getMACHints(preferIp6?: boolean): Array<[string, string]>;
}

class Network {
    prefixToMask(bits: number, v6?: boolean): string | null;
    maskToPrefix(netmask: string, v6?: boolean): number | null;
    formatWifiEncryption(encryption: WifiEncryption): string | null;
    flushCache(): Promise<any>;
    getProtocol(protoname: string, netname?: string): Protocol | null;
    getProtocols(): Protocol[];
    registerProtocol(protoname: string, methods: Record<string, any>): typeof Protocol;
    registerPatternVirtualpath(pat: RegExp): void;
    registerErrorCode(code: string, message: string): boolean;
    addNetwork(name: string, options?: Record<string, string | string[]>): Promise<Protocol | null>;
    getNetwork(name: string): Promise<Protocol | null>;
    getNetworks(): Promise<Protocol[]>;
    deleteNetwork(name: string): Promise<boolean>;
    renameNetwork(oldName: string, newName: string): Promise<boolean>;
    getDevice(name: string): Promise<Device | null>;
    getDevices(): Promise<Device[]>;
    isIgnoredDevice(name: string): boolean;
    getWifiDevice(devname: string): Promise<WifiDevice | null>;
    getWifiDevices(): Promise<WifiDevice[]>;
    getWifiNetwork(netname: string): Promise<WifiNetwork | null>;
    getWifiNetworks(): Promise<WifiNetwork[]>;
    addWifiNetwork(options: Record<string, string | string[]>): Promise<WifiNetwork | null>;
    deleteWifiNetwork(netname: string): Promise<boolean>;
    getWANNetworks(): Promise<Protocol[]>;
    getWAN6Networks(): Promise<Protocol[]>;
    getSwitchTopologies(): Promise<Record<string, SwitchTopology>>;
    getIfnameOf(obj: Protocol | Device | WifiDevice | WifiNetwork | string): string | null;
    getDSLModemType(): Promise<string | null>;
    getHostHints(): Promise<Hosts>;
}

export const network: Network;
