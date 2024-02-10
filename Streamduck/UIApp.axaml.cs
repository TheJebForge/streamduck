// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Threading;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml;

namespace Streamduck;

public partial class UIApp : Application {
	public CancellationTokenSource? CancellationTokenSource;
	public App? StreamduckApp { get; set; }
	public Window? MainWindow { get; set; }

	public override void Initialize() {
		AvaloniaXamlLoader.Load(this);
	}

	public void OpenUI(object? sender, EventArgs eventArgs) {
		if (MainWindow!.IsVisible) MainWindow!.Hide();
		else MainWindow!.Show();
	}

	public void Exit(object? sender, EventArgs eventArgs) {
		CancellationTokenSource?.Cancel();
	}
}