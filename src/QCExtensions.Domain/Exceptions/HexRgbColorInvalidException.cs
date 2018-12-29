using System;

namespace QCExtensions.Domain.Exceptions
{
	public class HexRgbColorInvalidException : Exception
	{
		public HexRgbColorInvalidException(string hexRgbColor, Exception ex)
			: base($"Hex RGB Color \"{hexRgbColor}\" is invalid.", ex)
		{
		}
	}
}
