// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Collections.Generic;
using Streamduck.Actions;
using Streamduck.Data;
using Streamduck.Rendering;
using Streamduck.Socket;
using Streamduck.Triggers;

namespace Streamduck.Plugins;

/**
 * Allows to query all loaded plugins
 */
public interface IPluginQuery {
	IEnumerable<Plugin> AllPlugins();
	Plugin? SpecificPlugin(string name);
	IEnumerable<T> PluginsAssignableTo<T>() where T : class;

	IEnumerable<Namespaced<Driver>> AllDrivers();
	IEnumerable<Namespaced<Driver>> DriversByPlugin(string pluginName);
	Namespaced<Driver>? SpecificDriver(NamespacedName name);

	Namespaced<Driver>? SpecificDriver(string pluginName, string name) =>
		SpecificDriver(new NamespacedName(pluginName, name));

	NamespacedName DriverName(Driver driver);

	IEnumerable<Namespaced<PluginAction>> AllActions();
	IEnumerable<Namespaced<PluginAction>> ActionsByPlugin(string pluginName);
	Namespaced<PluginAction>? SpecificAction(NamespacedName name);

	Namespaced<PluginAction>? SpecificAction(string pluginName, string name) =>
		SpecificAction(new NamespacedName(pluginName, name));

	NamespacedName ActionName(PluginAction action);

	IEnumerable<Namespaced<Renderer>> AllRenderers();
	IEnumerable<Namespaced<Renderer>> RenderersByPlugin(string pluginName);
	Namespaced<Renderer>? SpecificRenderer(NamespacedName name);

	Namespaced<Renderer>? SpecificRenderer(string pluginName, string name) =>
		SpecificRenderer(new NamespacedName(pluginName, name));

	NamespacedName RendererName(Renderer renderer);

	Namespaced<Renderer>? DefaultRenderer();

	IEnumerable<Namespaced<Trigger>> AllTriggers();
	IEnumerable<Namespaced<Trigger>> TriggersByPlugin(string pluginName);
	Namespaced<Trigger>? SpecificTrigger(NamespacedName name);

	Namespaced<Trigger>? SpecificTrigger(string pluginName, string name) =>
		SpecificTrigger(new NamespacedName(pluginName, name));

	NamespacedName TriggerName(Trigger trigger);

	IEnumerable<Namespaced<SocketRequest>> AllSocketRequests();
	IEnumerable<Namespaced<SocketRequest>> SocketRequestsByPlugin(string pluginName);
	Namespaced<SocketRequest>? SpecificSocketRequest(NamespacedName name);

	Namespaced<SocketRequest>? SpecificSocketRequest(string pluginName, string name) =>
		SpecificSocketRequest(new NamespacedName(pluginName, name));

	NamespacedName SocketRequestName(SocketRequest socketRequest);
}