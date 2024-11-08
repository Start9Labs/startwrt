import { uci } from "./uci"; // 'require uci';

function initFirewallState() {
	return L.resolveDefault(uci.load('firewall'));
}

function parseEnum(s, values) {
	if (s == null)
		return null;

	s = String(s).toUpperCase();

	if (s == '')
		return null;

	for (var i = 0; i < values.length; i++)
		if (values[i].toUpperCase().indexOf(s) == 0)
			return values[i];

	return null;
}

function parsePolicy(s, defaultValue) {
	return parseEnum(s, ['DROP', 'REJECT', 'ACCEPT']) || (arguments.length < 2 ? null : defaultValue);
}

function lookupZone(name) {
	var z = uci.get('firewall', name);

	if (z != null && z['.type'] == 'zone')
		return new Zone(z['.name']);

	var sections = uci.sections('firewall', 'zone');

	for (var i = 0; i < sections.length; i++) {
		if (sections[i].name != name)
			continue;

		return new Zone(sections[i]['.name']);
	}

	return null;
}

function getColorForName(forName) {
	if (forName == null)
		return '#eeeeee';
	else if (forName == 'lan')
		return '#90f090';
	else if (forName == 'wan')
		return '#f09090';

	return random.derive_color(forName);
}


export class Firewall {
	getDefaults() {
		return initFirewallState().then(function() {
			return new Defaults();
		});
	}

	newZone() {
		return initFirewallState().then(L.bind(function() {
			var name = 'newzone',
			    count = 1;

			while (this.getZone(name) != null)
				name = 'newzone%d'.format(++count);

			return this.addZone(name);
		}, this));
	}

	addZone(name) {
		return initFirewallState().then(L.bind(function() {
			if (name == null || !/^[a-zA-Z0-9_]+$/.test(name))
				return null;

			if (lookupZone(name) != null)
				return null;

			var d = new Defaults(),
			    z = uci.add('firewall', 'zone');

			uci.set('firewall', z, 'name',    name);
			uci.set('firewall', z, 'input',   d.getInput()   || 'DROP');
			uci.set('firewall', z, 'output',  d.getOutput()  || 'DROP');
			uci.set('firewall', z, 'forward', d.getForward() || 'DROP');

			return new Zone(z);
		}, this));
	}

	getZone(name) {
		return initFirewallState().then(function() {
			return lookupZone(name);
		});
	}

	getZones() {
		return initFirewallState().then(function() {
			var sections = uci.sections('firewall', 'zone'),
			    zones = [];

			for (var i = 0; i < sections.length; i++)
				zones.push(new Zone(sections[i]['.name']));

			zones.sort(function(a, b) { return a.getName() > b.getName() });

			return zones;
		});
	}

	getZoneByNetwork(network) {
		return initFirewallState().then(function() {
			var sections = uci.sections('firewall', 'zone');

			for (var i = 0; i < sections.length; i++)
				if (L.toArray(sections[i].network).indexOf(network) != -1)
					return new Zone(sections[i]['.name']);

			return null;
		});
	}

	deleteZone(name) {
		return initFirewallState().then(function() {
			var section = uci.get('firewall', name),
			    found = false;

			if (section != null && section['.type'] == 'zone') {
				found = true;
				name = section.name;
				uci.remove('firewall', section['.name']);
			}
			else if (name != null) {
				var sections = uci.sections('firewall', 'zone');

				for (var i = 0; i < sections.length; i++) {
					if (sections[i].name != name)
						continue;

					found = true;
					uci.remove('firewall', sections[i]['.name']);
				}
			}

			if (found == true) {
				sections = uci.sections('firewall');

				for (var i = 0; i < sections.length; i++) {
					if (sections[i]['.type'] != 'rule' &&
					    sections[i]['.type'] != 'redirect' &&
					    sections[i]['.type'] != 'forwarding')
					    continue;

					if (sections[i].src == name || sections[i].dest == name)
						uci.remove('firewall', sections[i]['.name']);
				}
			}

			return found;
		});
	}

	renameZone(oldName, newName) {
		return initFirewallState().then(L.bind(function() {
			if (oldName == null || newName == null || !/^[a-zA-Z0-9_]+$/.test(newName))
				return false;

			if (lookupZone(newName) != null)
				return false;

			var sections = uci.sections('firewall', 'zone'),
			    found = false;

			for (var i = 0; i < sections.length; i++) {
				if (sections[i].name != oldName)
					continue;

				uci.set('firewall', sections[i]['.name'], 'name', newName);
				found = true;
			}

			if (found == true) {
				sections = uci.sections('firewall');

				for (var i = 0; i < sections.length; i++) {
					if (sections[i]['.type'] != 'rule' &&
					    sections[i]['.type'] != 'redirect' &&
					    sections[i]['.type'] != 'forwarding')
					    continue;

					if (sections[i].src == oldName)
						uci.set('firewall', sections[i]['.name'], 'src', newName);

					if (sections[i].dest == oldName)
						uci.set('firewall', sections[i]['.name'], 'dest', newName);
				}
			}

			return found;
		}, this));
	}

	deleteNetwork(network) {
		return this.getZones().then(L.bind(function(zones) {
			var rv = false;

			for (var i = 0; i < zones.length; i++)
				if (zones[i].deleteNetwork(network))
					rv = true;

			return rv;
		}, this));
	}

	getColorForName = getColorForName;

	getZoneColorStyle(zone) {
		var hex = (zone instanceof Zone) ? zone.getColor() : getColorForName((zone != null && zone != '*') ? zone : null);

		return '--zone-color-rgb:%d, %d, %d; background-color:rgb(var(--zone-color-rgb))'.format(
			parseInt(hex.substring(1, 3), 16),
			parseInt(hex.substring(3, 5), 16),
			parseInt(hex.substring(5, 7), 16)
		);
	}
}


export class AbstractFirewallItem {
	get(option) {
		return uci.get('firewall', this.sid, option);
	}

	set(option, value) {
		return uci.set('firewall', this.sid, option, value);
	}
}


export class Defaults extends AbstractFirewallItem {
	constructor() {
		var sections = uci.sections('firewall', 'defaults');

		for (var i = 0; i < sections.length; i++) {
			this.sid = sections[i]['.name'];
			break;
		}

		if (this.sid == null)
			this.sid = uci.add('firewall', 'defaults');
	}

	isSynFlood() {
		return (this.get('syn_flood') == '1');
	}

	isDropInvalid() {
		return (this.get('drop_invalid') == '1');
	}

	getInput() {
		return parsePolicy(this.get('input'), 'DROP');
	}

	getOutput() {
		return parsePolicy(this.get('output'), 'DROP');
	}

	getForward() {
		return parsePolicy(this.get('forward'), 'DROP');
	}
}

export class Zone extends AbstractFirewallItem {
	constructor(name) {
		var section = uci.get('firewall', name);

		if (section != null && section['.type'] == 'zone') {
			this.sid  = name;
			this.data = section;
		}
		else if (name != null) {
			var sections = uci.get('firewall', 'zone');

			for (var i = 0; i < sections.length; i++) {
				if (sections[i].name != name)
					continue;

				this.sid  = sections[i]['.name'];
				this.data = sections[i];
				break;
			}
		}
	}

	isMasquerade() {
		return (this.get('masq') == '1');
	}

	getName() {
		return this.get('name');
	}

	getNetwork() {
		return this.get('network');
	}

	getInput() {
		return parsePolicy(this.get('input'), (new Defaults()).getInput());
	}

	getOutput() {
		return parsePolicy(this.get('output'), (new Defaults()).getOutput());
	}

	getForward() {
		return parsePolicy(this.get('forward'), (new Defaults()).getForward());
	}

	addNetwork(network) {
		var section = uci.get('network', network);

		if (section == null || section['.type'] != 'interface')
			return false;

		var newNetworks = this.getNetworks();

		if (newNetworks.filter(function(net) { return net == network }).length)
			return false;

		newNetworks.push(network);
		this.set('network', newNetworks);

		return true;
	}

	deleteNetwork(network) {
		var oldNetworks = this.getNetworks(),
		    newNetworks = oldNetworks.filter(function(net) { return net != network });

		if (newNetworks.length > 0)
			this.set('network', newNetworks);
		else
			this.set('network', null);

		return (newNetworks.length < oldNetworks.length);
	}

	getNetworks() {
		return L.toArray(this.get('network'));
	}

	clearNetworks() {
		this.set('network', null);
	}

	getDevices() {
		return L.toArray(this.get('device'));
	}

	getSubnets() {
		return L.toArray(this.get('subnet'));
	}

	getForwardingsBy(what) {
		var sections = uci.sections('firewall', 'forwarding'),
		    forwards = [];

		for (var i = 0; i < sections.length; i++) {
			if (sections[i].src == null || sections[i].dest == null)
				continue;

			if (sections[i][what] != this.getName())
				continue;

			forwards.push(new Forwarding(sections[i]['.name']));
		}

		return forwards;
	}

	addForwardingTo(dest) {
		var forwards = this.getForwardingsBy('src'),
		    zone = lookupZone(dest);

		if (zone == null || zone.getName() == this.getName())
			return null;

		for (var i = 0; i < forwards.length; i++)
			if (forwards[i].getDestination() == zone.getName())
				return null;

		var sid = uci.add('firewall', 'forwarding');

		uci.set('firewall', sid, 'src', this.getName());
		uci.set('firewall', sid, 'dest', zone.getName());

		return new Forwarding(sid);
	}

	addForwardingFrom(src) {
		var forwards = this.getForwardingsBy('dest'),
		    zone = lookupZone(src);

		if (zone == null || zone.getName() == this.getName())
			return null;

		for (var i = 0; i < forwards.length; i++)
			if (forwards[i].getSource() == zone.getName())
				return null;

		var sid = uci.add('firewall', 'forwarding');

		uci.set('firewall', sid, 'src', zone.getName());
		uci.set('firewall', sid, 'dest', this.getName());

		return new Forwarding(sid);
	}

	deleteForwardingsBy(what) {
		var sections = uci.sections('firewall', 'forwarding'),
		    found = false;

		for (var i = 0; i < sections.length; i++) {
			if (sections[i].src == null || sections[i].dest == null)
				continue;

			if (sections[i][what] != this.getName())
				continue;

			uci.remove('firewall', sections[i]['.name']);
			found = true;
		}

		return found;
	}

	deleteForwarding(forwarding) {
		if (!(forwarding instanceof Forwarding))
			return false;

		var section = uci.get('firewall', forwarding.sid);

		if (!section || section['.type'] != 'forwarding')
			return false;

		uci.remove('firewall', section['.name']);

		return true;
	}

	addRedirect(options) {
		var sid = uci.add('firewall', 'redirect');

		if (options != null && typeof(options) == 'object')
			for (var key in options)
				if (options.hasOwnProperty(key))
					uci.set('firewall', sid, key, options[key]);

		uci.set('firewall', sid, 'src', this.getName());

		return new Redirect(sid);
	}

	addRule(options) {
		var sid = uci.add('firewall', 'rule');

		if (options != null && typeof(options) == 'object')
			for (var key in options)
				if (options.hasOwnProperty(key))
					uci.set('firewall', sid, key, options[key]);

		uci.set('firewall', sid, 'src', this.getName());

		return new Rule(sid);
	}

	getColor(forName) {
		var name = (arguments.length > 0 ? forName : this.getName());

		return getColorForName(name);
	}
}

export class Forwarding extends AbstractFirewallItem {
	constructor(sid) {
		this.sid = sid;
	}

	getSource() {
		return this.get('src');
	}

	getDestination() {
		return this.get('dest');
	}

	getSourceZone() {
		return lookupZone(this.getSource());
	}

	getDestinationZone() {
		return lookupZone(this.getDestination());
	}
}


export class Rule extends AbstractFirewallItem {
	getSource() {
		return this.get('src');
	}

	getDestination() {
		return this.get('dest');
	}

	getSourceZone() {
		return lookupZone(this.getSource());
	}

	getDestinationZone() {
		return lookupZone(this.getDestination());
	}
}


export class Redirect {
	getSource() {
		return this.get('src');
	}

	getDestination() {
		return this.get('dest');
	}

	getSourceZone() {
		return lookupZone(this.getSource());
	}

	getDestinationZone() {
		return lookupZone(this.getDestination());
	}
}


export const firewall = new Firewall();
