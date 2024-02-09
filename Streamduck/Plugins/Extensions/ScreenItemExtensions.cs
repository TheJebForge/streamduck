using System.Reflection;
using Streamduck.Cores;
using Streamduck.Cores.ScreenItems;
using Streamduck.Data;
using Streamduck.Rendering;

namespace Streamduck.Plugins.Extensions;

public static class ScreenItemExtensions {
	public static ScreenItem ReplaceRenderer(this ScreenItem item, Namespaced<Renderer> renderer) {
		if (item is not ScreenItem.IRenderable renderable) return item;

		if (!(renderer.Instance.GetType().BaseType?.IsGenericType ?? false)
		    || renderer.Instance.GetType().BaseType!.GetGenericTypeDefinition() != typeof(Renderer<>)) {
			return new RenderableScreenItem(null, item.Triggers) {
				AssociatedInput = item is ScreenlessItem screenless ? screenless.AssociatedInput : null,
				RendererName = renderer.NamespacedName,
				RendererSettings = renderer.Instance.DefaultRendererConfig
			};
		}
		
		var typeArgument = renderer.Instance.GetType().BaseType!.GetGenericArguments()[0];
		return (ScreenItem) GenericRendererMethod.MakeGenericMethod(typeArgument)
			.Invoke(null, [item, renderer])!;
	}

	private const BindingFlags StaticNonPublic = BindingFlags.Static | BindingFlags.NonPublic;
	private static readonly MethodInfo GenericRendererMethod =
		typeof(ScreenItemExtensions).GetMethod(nameof(GenericRenderer), StaticNonPublic)!;

	private static TypedRenderableScreenItem<T> GenericRenderer<T>(ScreenItem item, Namespaced<Renderer<T>> renderer)
		where T : class, new() =>
		new(null, item.Triggers) {
			AssociatedInput = item is ScreenlessItem screenless ? screenless.AssociatedInput : null,
			RendererName = renderer.NamespacedName,
			RendererSettings = (T) renderer.Instance.DefaultRendererConfig
		};
}