using System.Threading.Tasks;
using SixLabors.ImageSharp;
using Streamduck.Data;

namespace Streamduck.Inputs;

public interface IInputDisplay {
	UInt2 DisplayResolution { get; }

	/**
	 * Streamduck will hash its render structures and then call this to append the hash in case the devices requires
	 * 2 different formats depending on which input is being rendered to
	 */
	long AppendHashKey(long key);

	/**
	 * Lets the device process the image into format it needs,
	 * key is derived from render structure and appended by the input
	 */
	Task UploadImage(long key, Image image);

	/**
	 * Should return true if image still exists, false if image was already deleted by the cache
	 */
	ValueTask<bool> ApplyImage(long key);
}