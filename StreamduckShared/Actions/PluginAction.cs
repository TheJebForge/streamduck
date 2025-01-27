// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Threading.Tasks;
using Streamduck.Interfaces;

namespace Streamduck.Actions;

/**
 * Action that can be triggered by Triggers. Data won't be saved since the type is unknown, use typed version for data loading
 */
public abstract class PluginAction : INamed {
	public abstract string? Description { get; }
	public abstract string Name { get; }

	/**
	 * <exception cref="System.ArgumentException">If argument was of invalid type</exception>
	 */
	public abstract Task Invoke(object data);

	/**
	 * Should create default data that can be used with Invoke
	 */
	public abstract Task<object> DefaultData();
}

/**
 * Action that can be triggered by Triggers, but also has typed data associated
 */
public abstract class PluginAction<T> : PluginAction where T : class, new() {
	public override Task Invoke(object data) {
		if (data is not T casted)
			throw new ArgumentException($"Data is of type {data.GetType()}, expected {typeof(T)}");

		return Invoke(casted);
	}

	public abstract Task Invoke(T data);

	public override Task<object> DefaultData() => Task.FromResult((object)new T());
}