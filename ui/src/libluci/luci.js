export var env = {
	scriptname: 'http://192.168.122.192',
}

/**
 * @class LuCI
 * @classdesc
 *
 * This is the LuCI base class. It is automatically instantiated and
 * accessible using the global `L` variable.
 *
 * @param {Object} env
 * The environment settings to use for the LuCI runtime.
 */
class LuCI {
		/**
		 * Captures the current stack trace and throws an error of thei
		 * specified type as a new exception. Also logs the exception as
		 * error to the debug console if it is available.
		 *
		 * @instance
		 * @memberof LuCI
		 *
		 * @param {Error|string} [type=Error]
		 * Either a string specifying the type of the error to throw or an
		 * existing `Error` instance to copy.
		 *
		 * @param {string} [fmt=Unspecified error]
		 * A format string which is used to form the error message, together
		 * with all subsequent optional arguments.
		 *
		 * @param {...*} [args]
		 * Zero or more variable arguments to the supplied format string.
		 *
		 * @throws {Error}
		 * Throws the created error object with the captured stack trace
		 * appended to the message and the type set to the given type
		 * argument or copied from the given error instance.
		 */
		raise(type, fmt, ...varargs) {
			var e = null,
			    msg = fmt ? String.prototype.format.apply(fmt, varargs) : null,
			    stack = null;

			if (type instanceof Error) {
				e = type;

				if (msg)
					e.message = msg + ': ' + e.message;
			}
			else {
				try { throw new Error('stacktrace') }
				catch (e2) { stack = (e2.stack || '').split(/\n/) }

				e = new (window[type || 'Error'] || Error)(msg || 'Unspecified error');
				e.name = type || 'Error';
			}

			stack = (stack || []).map(function(frame) {
				frame = frame.replace(/(.*?)@(.+):(\d+):(\d+)/g, 'at $1 ($2:$3:$4)').trim();
				return frame ? '  ' + frame : '';
			});

			if (!/^  at /.test(stack[0]))
				stack.shift();

			if (/\braise /.test(stack[0]))
				stack.shift();

			if (/\berror /.test(stack[0]))
				stack.shift();

			if (stack.length)
				e.message += '\n' + stack.join('\n');

			if (window.console && console.debug)
				console.debug(e);

			throw e;
		}

		/**
		 * A wrapper around {@link LuCI#raise raise()} which also renders
		 * the error either as modal overlay when `ui.js` is already loaed
		 * or directly into the view body.
		 *
		 * @instance
		 * @memberof LuCI
		 *
		 * @param {Error|string} [type=Error]
		 * Either a string specifying the type of the error to throw or an
		 * existing `Error` instance to copy.
		 *
		 * @param {string} [fmt=Unspecified error]
		 * A format string which is used to form the error message, together
		 * with all subsequent optional arguments.
		 *
		 * @param {...*} [args]
		 * Zero or more variable arguments to the supplied format string.
		 *
		 * @throws {Error}
		 * Throws the created error object with the captured stack trace
		 * appended to the message and the type set to the given type
		 * argument or copied from the given error instance.
		 */
		error(type, fmt /*, ...*/) {
			try {
				LuCI.prototype.raise.apply(LuCI.prototype,
					Array.prototype.slice.call(arguments));
			}
			catch (e) {
				if (!e.reported) {
					if (classes.ui)
						classes.ui.addNotification(e.name || _('Runtime error'),
							E('pre', {}, e.message), 'danger');
					else
						DOM.content(document.querySelector('#maincontent'),
							E('pre', { 'class': 'alert-message error' }, e.message));

					e.reported = true;
				}

				throw e;
			}
		}

		/**
		 * Return a bound function using the given `self` as `this` context
		 * and any further arguments as parameters to the bound function.
		 *
		 * @instance
		 * @memberof LuCI
		 *
		 * @param {function} fn
		 * The function to bind.
		 *
		 * @param {*} self
		 * The value to bind as `this` context to the specified function.
		 *
		 * @param {...*} [args]
		 * Zero or more variable arguments which are bound to the function
		 * as parameters.
		 *
		 * @returns {function}
		 * Returns the bound function.
		 */
		bind(fn, self, ...varargs) {
			return Function.prototype.bind.apply(fn, varargs);
		}

		/**
		 * Load an additional LuCI JavaScript class and its dependencies,
		 * instantiate it and return the resulting class instance. Each
		 * class is only loaded once. Subsequent attempts to load the same
		 * class will return the already instantiated class.
		 *
		 * @instance
		 * @memberof LuCI
		 *
		 * @param {string} name
		 * The name of the class to load in dotted notation. Dots will
		 * be replaced by spaces and joined with the runtime-determined
		 * base URL of LuCI.js to form an absolute URL to load the class
		 * file from.
		 *
		 * @throws {DependencyError}
		 * Throws a `DependencyError` when the class to load includes
		 * circular dependencies.
		 *
		 * @throws {NetworkError}
		 * Throws `NetworkError` when the underlying {@link LuCI.request}
		 * call failed.
		 *
		 * @throws {SyntaxError}
		 * Throws `SyntaxError` when the loaded class file code cannot
		 * be interpreted by `eval`.
		 *
		 * @throws {TypeError}
		 * Throws `TypeError` when the class file could be loaded and
		 * interpreted, but when invoking its code did not yield a valid
		 * class instance.
		 *
		 * @returns {Promise<LuCI.baseclass>}
		 * Returns the instantiated class.
		 */
		require(name, from) {
			throw Error('startwrt does not use the luci require polyfill');
		}

		/* DOM setup */
		probeRPCBaseURL() {
			if (rpcBaseURL == null)
				rpcBaseURL = Session.getLocalData('rpcBaseURL');

			if (rpcBaseURL == null) {
				var msg = {
					jsonrpc: '2.0',
					id:      'init',
					method:  'list',
					params:  undefined
				};
				var rpcFallbackURL = this.url('admin/ubus');

				rpcBaseURL = Request.post(env.ubuspath, msg, { nobatch: true }).then(function(res) {
					return (rpcBaseURL = res.status == 200 ? env.ubuspath : rpcFallbackURL);
				}, function() {
					return (rpcBaseURL = rpcFallbackURL);
				}).then(function(url) {
					Session.setLocalData('rpcBaseURL', url);
					return url;
				});
			}

			return Promise.resolve(rpcBaseURL);
		}

		probeSystemFeatures() {
			if (sysFeatures == null)
				sysFeatures = Session.getLocalData('features');

			if (!this.isObject(sysFeatures)) {
				sysFeatures = classes.rpc.declare({
					object: 'luci',
					method: 'getFeatures',
					expect: { '': {} }
				})().then(function(features) {
					Session.setLocalData('features', features);
					sysFeatures = features;

					return features;
				});
			}

			return Promise.resolve(sysFeatures);
		}

		probePreloadClasses() {
			if (preloadClasses == null)
				preloadClasses = Session.getLocalData('preload');

			if (!Array.isArray(preloadClasses)) {
				preloadClasses = this.resolveDefault(classes.rpc.declare({
					object: 'file',
					method: 'list',
					params: [ 'path' ],
					expect: { 'entries': [] }
				})(this.fspath(this.resource('preload'))), []).then(function(entries) {
					var classes = [];

					for (var i = 0; i < entries.length; i++) {
						if (entries[i].type != 'file')
							continue;

						var m = entries[i].name.match(/(.+)\.js$/);

						if (m)
							classes.push('preload.%s'.format(m[1]));
					}

					Session.setLocalData('preload', classes);
					preloadClasses = classes;

					return classes;
				});
			}

			return Promise.resolve(preloadClasses);
		}

		/**
		 * Test whether a particular system feature is available, such as
		 * hostapd SAE support or an installed firewall. The features are
		 * queried once at the beginning of the LuCI session and cached in
		 * `SessionStorage` throughout the lifetime of the associated tab or
		 * browser window.
		 *
		 * @instance
		 * @memberof LuCI
		 *
		 * @param {string} feature
		 * The feature to test. For detailed list of known feature flags,
		 * see `/modules/luci-base/root/usr/libexec/rpcd/luci`.
		 *
		 * @param {string} [subfeature]
		 * Some feature classes like `hostapd` provide sub-feature flags,
		 * such as `sae` or `11w` support. The `subfeature` argument can
		 * be used to query these.
		 *
		 * @return {boolean|null}
		 * Return `true` if the queried feature (and sub-feature) is available
		 * or `false` if the requested feature isn't present or known.
		 * Return `null` when a sub-feature was queried for a feature which
		 * has no sub-features.
		 */
		hasSystemFeature() {
			var ft = sysFeatures[arguments[0]];

			if (arguments.length == 2)
				return this.isObject(ft) ? ft[arguments[1]] : null;

			return (ft != null && ft != false);
		}

		/* private */
		notifySessionExpiry() {
			Poll.stop();

			classes.ui.showModal(_('Session expired'), [
				E('div', { class: 'alert-message warning' },
					_('A new login is required since the authentication session expired.')),
				E('div', { class: 'right' },
					E('div', {
						class: 'btn primary',
						click() {
							var loc = window.location;
							window.location = loc.protocol + '//' + loc.host + loc.pathname + loc.search;
						}
					}, _('Log in…')))
			]);

			LuCI.prototype.raise('SessionError', 'Login session is expired');
		}

		/* private */
		setupDOM(res) {
			var domEv = res[0],
			    uiClass = res[1],
			    rpcClass = res[2],
			    formClass = res[3],
			    rpcBaseURL = res[4];

			rpcClass.setBaseURL(rpcBaseURL);

			rpcClass.addInterceptor(function(msg, req) {
				if (!LuCI.prototype.isObject(msg) ||
				    !LuCI.prototype.isObject(msg.error) ||
				    msg.error.code != -32002)
					return;

				if (!LuCI.prototype.isObject(req) ||
				    (req.object == 'session' && req.method == 'access'))
					return;

				return rpcClass.declare({
					'object': 'session',
					'method': 'access',
					'params': [ 'scope', 'object', 'function' ],
					'expect': { access: true }
				})('uci', 'luci', 'read').catch(LuCI.prototype.notifySessionExpiry);
			});

			Request.addInterceptor(function(res) {
				var isDenied = false;

				if (res.status == 403 && res.headers.get('X-LuCI-Login-Required') == 'yes')
					isDenied = true;

				if (!isDenied)
					return;

				LuCI.prototype.notifySessionExpiry();
			});

			document.addEventListener('poll-start', function(ev) {
				uiClass.showIndicator('poll-status', _('Refreshing'), function(ev) {
					Request.poll.active() ? Request.poll.stop() : Request.poll.start();
				});
			});

			document.addEventListener('poll-stop', function(ev) {
				uiClass.showIndicator('poll-status', _('Paused'), null, 'inactive');
			});

			return Promise.all([
				this.probeSystemFeatures(),
				this.probePreloadClasses()
			]).finally(LuCI.prototype.bind(function() {
				var tasks = [];

				if (Array.isArray(preloadClasses))
					for (var i = 0; i < preloadClasses.length; i++)
						tasks.push(this.require(preloadClasses[i]));

				return Promise.all(tasks);
			}, this)).finally(this.initDOM);
		}

		/* private */
		initDOM() {
			originalCBIInit();
			Poll.start();
			document.dispatchEvent(new CustomEvent('luci-loaded'));
		}

		/**
		 * The `env` object holds environment settings used by LuCI, such
		 * as request timeouts, base URLs etc.
		 *
		 * @instance
		 * @memberof LuCI
		 */
		env=env

		/**
		 * Construct an absolute filesystem path relative to the server
		 * document root.
		 *
		 * @instance
		 * @memberof LuCI
		 *
		 * @param {...string} [parts]
		 * An array of parts to join into a path.
		 *
		 * @return {string}
		 * Return the joined path.
		 */
		fspath(/* ... */) {
			var path = env.documentroot;

			for (var i = 0; i < arguments.length; i++)
				path += '/' + arguments[i];

			var p = path.replace(/\/+$/, '').replace(/\/+/g, '/').split(/\//),
			    res = [];

			for (var i = 0; i < p.length; i++)
				if (p[i] == '..')
					res.pop();
				else if (p[i] != '.')
					res.push(p[i]);

			return res.join('/');
		}

		/**
		 * Construct a relative URL path from the given prefix and parts.
		 * The resulting URL is guaranteed to only contain the characters
		 * `a-z`, `A-Z`, `0-9`, `_`, `.`, `%`, `,`, `;`, and `-` as well
		 * as `/` for the path separator.
		 *
		 * @instance
		 * @memberof LuCI
		 *
		 * @param {string} [prefix]
		 * The prefix to join the given parts with. If the `prefix` is
		 * omitted, it defaults to an empty string.
		 *
		 * @param {string[]} [parts]
		 * An array of parts to join into an URL path. Parts may contain
		 * slashes and any of the other characters mentioned above.
		 *
		 * @return {string}
		 * Return the joined URL path.
		 */
		path(prefix, parts) {
			var url = [ prefix || '' ];

			for (var i = 0; i < parts.length; i++)
				if (/^(?:[a-zA-Z0-9_.%,;-]+\/)*[a-zA-Z0-9_.%,;-]+$/.test(parts[i]))
					url.push('/', parts[i]);

			if (url.length === 1)
				url.push('/');

			return url.join('');
		}

		/**
		 * Construct an URL  pathrelative to the script path of the server
		 * side LuCI application (usually `/cgi-bin/luci`).
		 *
		 * The resulting URL is guaranteed to only contain the characters
		 * `a-z`, `A-Z`, `0-9`, `_`, `.`, `%`, `,`, `;`, and `-` as well
		 * as `/` for the path separator.
		 *
		 * @instance
		 * @memberof LuCI
		 *
		 * @param {string[]} [parts]
		 * An array of parts to join into an URL path. Parts may contain
		 * slashes and any of the other characters mentioned above.
		 *
		 * @return {string}
		 * Returns the resulting URL path.
		 */
		url() {
			return this.path(env.scriptname, arguments);
		}

		/**
		 * Construct an URL path relative to the global static resource path
		 * of the LuCI ui (usually `/luci-static/resources`).
		 *
		 * The resulting URL is guaranteed to only contain the characters
		 * `a-z`, `A-Z`, `0-9`, `_`, `.`, `%`, `,`, `;`, and `-` as well
		 * as `/` for the path separator.
		 *
		 * @instance
		 * @memberof LuCI
		 *
		 * @param {string[]} [parts]
		 * An array of parts to join into an URL path. Parts may contain
		 * slashes and any of the other characters mentioned above.
		 *
		 * @return {string}
		 * Returns the resulting URL path.
		 */
		resource() {
			return this.path(env.resource, arguments);
		}

		/**
		 * Construct an URL path relative to the media resource path of the
		 * LuCI ui (usually `/luci-static/$theme_name`).
		 *
		 * The resulting URL is guaranteed to only contain the characters
		 * `a-z`, `A-Z`, `0-9`, `_`, `.`, `%`, `,`, `;`, and `-` as well
		 * as `/` for the path separator.
		 *
		 * @instance
		 * @memberof LuCI
		 *
		 * @param {string[]} [parts]
		 * An array of parts to join into an URL path. Parts may contain
		 * slashes and any of the other characters mentioned above.
		 *
		 * @return {string}
		 * Returns the resulting URL path.
		 */
		media() {
			return this.path(env.media, arguments);
		}

		/**
		 * Return the complete URL path to the current view.
		 *
		 * @instance
		 * @memberof LuCI
		 *
		 * @return {string}
		 * Returns the URL path to the current view.
		 */
		location() {
			return this.path(env.scriptname, env.requestpath);
		}


		/**
		 * Tests whether the passed argument is a JavaScript object.
		 * This function is meant to be an object counterpart to the
		 * standard `Array.isArray()` function.
		 *
		 * @instance
		 * @memberof LuCI
		 *
		 * @param {*} [val]
		 * The value to test
		 *
		 * @return {boolean}
		 * Returns `true` if the given value is of type object and
		 * not `null`, else returns `false`.
		 */
		isObject(val) {
			return (val != null && typeof(val) == 'object');
		}

		/**
		 * Return an array of sorted object keys, optionally sorted by
		 * a different key or a different sorting mode.
		 *
		 * @instance
		 * @memberof LuCI
		 *
		 * @param {object} obj
		 * The object to extract the keys from. If the given value is
		 * not an object, the function will return an empty array.
		 *
		 * @param {string} [key]
		 * Specifies the key to order by. This is mainly useful for
		 * nested objects of objects or objects of arrays when sorting
		 * shall not be performed by the primary object keys but by
		 * some other key pointing to a value within the nested values.
		 *
		 * @param {string} [sortmode]
		 * May be either `addr` or `num` to override the natural
		 * lexicographic sorting with a sorting suitable for IP/MAC style
		 * addresses or numeric values respectively.
		 *
		 * @return {string[]}
		 * Returns an array containing the sorted keys of the given object.
		 */
		sortedKeys(obj, key, sortmode) {
			if (obj == null || typeof(obj) != 'object')
				return [];

			return Object.keys(obj).map(function(e) {
				var v = (key != null) ? obj[e][key] : e;

				switch (sortmode) {
				case 'addr':
					v = (v != null) ? v.replace(/(?:^|[.:])([0-9a-fA-F]{1,4})/g,
						function(m0, m1) { return ('000' + m1.toLowerCase()).substr(-4) }) : null;
					break;

				case 'num':
					v = (v != null) ? +v : null;
					break;
				}

				return [ e, v ];
			}).filter(function(e) {
				return (e[1] != null);
			}).sort(function(a, b) {
				return naturalCompare(a[1], b[1]);
			}).map(function(e) {
				return e[0];
			});
		}

		/**
		 * Compares two values numerically and returns -1, 0 or 1 depending
		 * on whether the first value is smaller, equal to or larger than the
		 * second one respectively.
		 *
		 * This function is meant to be used as comparator function for
		 * Array.sort().
		 *
		 * @type {function}
		 *
		 * @param {*} a
		 * The first value
		 *
		 * @param {*} b
		 * The second value.
		 *
		 * @return {number}
		 * Returns -1 if the first value is smaller than the second one.
		 * Returns 0 if both values are equal.
		 * Returns 1 if the first value is larger than the second one.
		 */
		naturalCompare = new Intl.Collator(undefined, { numeric: true }).compare;

		/**
		 * Converts the given value to an array using toArray() if needed,
		 * performs a numerical sort using naturalCompare() and returns the
		 * result. If the input already is an array, no copy is being made
		 * and the sorting is performed in-place.
		 *
		 * @see toArray
		 * @see naturalCompare
		 *
		 * @param {*} val
		 * The input value to sort (and convert to an array if needed).
		 *
		 * @return {Array<*>}
		 * Returns the resulting, numerically sorted array.
		 */
		sortedArray(val) {
			return this.toArray(val).sort(naturalCompare);
		}

		/**
		 * Converts the given value to an array. If the given value is of
		 * type array, it is returned as-is, values of type object are
		 * returned as one-element array containing the object, empty
		 * strings and `null` values are returned as empty array, all other
		 * values are converted using `String()`, trimmed, split on white
		 * space and returned as array.
		 *
		 * @instance
		 * @memberof LuCI
		 *
		 * @param {*} val
		 * The value to convert into an array.
		 *
		 * @return {Array<*>}
		 * Returns the resulting array.
		 */
		toArray(val) {
			if (val == null)
				return [];
			else if (Array.isArray(val))
				return val;
			else if (typeof(val) == 'object')
				return [ val ];

			var s = String(val).trim();

			if (s == '')
				return [];

			return s.split(/\s+/);
		}

		/**
		 * Returns a promise resolving with either the given value or or with
		 * the given default in case the input value is a rejecting promise.
		 *
		 * @instance
		 * @memberof LuCI
		 *
		 * @param {*} value
		 * The value to resolve the promise with.
		 *
		 * @param {*} defvalue
		 * The default value to resolve the promise with in case the given
		 * input value is a rejecting promise.
		 *
		 * @returns {Promise<*>}
		 * Returns a new promise resolving either to the given input value or
		 * to the given default value on error.
		 */
		resolveDefault(value, defvalue) {
			return Promise.resolve(value).catch(function() { return defvalue });
		}

		/**
		 * Check whether a view has sufficient permissions.
		 *
		 * @return {boolean|null}
		 * Returns `null` if the current session has no permission at all to
		 * load resources required by the view. Returns `false` if readonly
		 * permissions are granted or `true` if at least one required ACL
		 * group is granted with write permissions.
		 */
		hasViewPermission() {
			if (!this.isObject(env.nodespec) || !env.nodespec.satisfied)
			    return null;

			return !env.nodespec.readonly;
		}
}

export const L = new LuCI();
