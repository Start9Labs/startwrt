import { SectionObject } from './uci';

export interface FirewallOptions {
    [key: string]: string | string[] | null;
}

export type PolicyType = 'DROP' | 'REJECT' | 'ACCEPT' | null;

export class Firewall {
    /**
     * Gets the default firewall settings
     */
    getDefaults(): Promise<Defaults>;

    /**
     * Creates a new zone with an automatically generated name
     */
    newZone(): Promise<Zone | null>;

    /**
     * Adds a new zone with the specified name
     * @param name - Name for the new zone
     */
    addZone(name: string): Promise<Zone | null>;

    /**
     * Gets a zone by name
     * @param name - Name of the zone to retrieve
     */
    getZone(name: string): Promise<Zone | null>;

    /**
     * Gets all configured zones
     */
    getZones(): Promise<Zone[]>;

    /**
     * Gets a zone by network name
     * @param network - Network name to look up
     */
    getZoneByNetwork(network: string): Promise<Zone | null>;

    /**
     * Deletes a zone by name
     * @param name - Name of the zone to delete
     */
    deleteZone(name: string): Promise<boolean>;

    /**
     * Renames an existing zone
     * @param oldName - Current name of the zone
     * @param newName - New name for the zone
     */
    renameZone(oldName: string, newName: string): Promise<boolean>;

    /**
     * Removes a network from all zones
     * @param network - Network name to remove
     */
    deleteNetwork(network: string): Promise<boolean>;

    /**
     * Gets the color associated with a zone name
     * @param name - Zone name
     */
    getColorForName(name: string | null): string;

    /**
     * Gets CSS style string for zone coloring
     * @param zone - Zone instance or name
     */
    getZoneColorStyle(zone: Zone | string | null): string;
}

export class AbstractFirewallItem {
    protected sid: string;

    /**
     * Gets an option value
     * @param option - Option name to get
     */
    get(option: string): string | string[] | null;

    /**
     * Sets an option value
     * @param option - Option name to set
     * @param value - Value to set
     */
    set(option: string, value: string | string[] | null): void;
}

export class Defaults extends AbstractFirewallItem {
    constructor();

    /**
     * Checks if SYN flood protection is enabled
     */
    isSynFlood(): boolean;

    /**
     * Checks if invalid packets should be dropped
     */
    isDropInvalid(): boolean;

    /**
     * Gets the default input policy
     */
    getInput(): PolicyType;

    /**
     * Gets the default output policy
     */
    getOutput(): PolicyType;

    /**
     * Gets the default forward policy
     */
    getForward(): PolicyType;
}

export class Zone extends AbstractFirewallItem {
    constructor(name: string);

    /**
     * Checks if masquerading is enabled
     */
    isMasquerade(): boolean;

    /**
     * Gets the zone name
     */
    getName(): string;

    /**
     * Gets the network configuration
     */
    getNetwork(): string | string[] | null;

    /**
     * Gets the input policy
     */
    getInput(): PolicyType;

    /**
     * Gets the output policy
     */
    getOutput(): PolicyType;

    /**
     * Gets the forward policy
     */
    getForward(): PolicyType;

    /**
     * Adds a network to the zone
     * @param network - Network name to add
     */
    addNetwork(network: string): boolean;

    /**
     * Removes a network from the zone
     * @param network - Network name to remove
     */
    deleteNetwork(network: string): boolean;

    /**
     * Gets all networks in the zone
     */
    getNetworks(): string[];

    /**
     * Removes all networks from the zone
     */
    clearNetworks(): void;

    /**
     * Gets configured devices
     */
    getDevices(): string[];

    /**
     * Gets configured subnets
     */
    getSubnets(): string[];

    /**
     * Gets forwardings by source or destination
     * @param what - 'src' or 'dest'
     */
    getForwardingsBy(what: 'src' | 'dest'): Forwarding[];

    /**
     * Adds forwarding to destination zone
     * @param dest - Destination zone name
     */
    addForwardingTo(dest: string): Forwarding | null;

    /**
     * Adds forwarding from source zone
     * @param src - Source zone name
     */
    addForwardingFrom(src: string): Forwarding | null;

    /**
     * Deletes forwardings by source or destination
     * @param what - 'src' or 'dest'
     */
    deleteForwardingsBy(what: 'src' | 'dest'): boolean;

    /**
     * Deletes a specific forwarding
     * @param forwarding - Forwarding instance to delete
     */
    deleteForwarding(forwarding: Forwarding): boolean;

    /**
     * Adds a redirect rule
     * @param options - Rule options
     */
    addRedirect(options?: FirewallOptions): Redirect;

    /**
     * Adds a firewall rule
     * @param options - Rule options
     */
    addRule(options?: FirewallOptions): Rule;

    /**
     * Gets the color for this zone or specified name
     * @param forName - Optional zone name to get color for
     */
    getColor(forName?: string): string;
}

export class Forwarding extends AbstractFirewallItem {
    constructor(sid: string);

    /**
     * Gets the source zone name
     */
    getSource(): string | null;

    /**
     * Gets the destination zone name
     */
    getDestination(): string | null;

    /**
     * Gets the source zone instance
     */
    getSourceZone(): Zone | null;

    /**
     * Gets the destination zone instance
     */
    getDestinationZone(): Zone | null;
}

export class Rule extends AbstractFirewallItem {
    /**
     * Gets the source zone name
     */
    getSource(): string | null;

    /**
     * Gets the destination zone name
     */
    getDestination(): string | null;

    /**
     * Gets the source zone instance
     */
    getSourceZone(): Zone | null;

    /**
     * Gets the destination zone instance
     */
    getDestinationZone(): Zone | null;
}

export class Redirect extends AbstractFirewallItem {
    /**
     * Gets the source zone name
     */
    getSource(): string | null;

    /**
     * Gets the destination zone name
     */
    getDestination(): string | null;

    /**
     * Gets the source zone instance
     */
    getSourceZone(): Zone | null;

    /**
     * Gets the destination zone instance
     */
    getDestinationZone(): Zone | null;
}

export const firewall: Firewall;
