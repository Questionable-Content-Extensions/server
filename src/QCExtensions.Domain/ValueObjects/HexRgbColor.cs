using System;
using System.Collections.Generic;
using System.Globalization;
using QCExtensions.Domain.Exceptions;
using QCExtensions.Domain.Infrastructure;

namespace QCExtensions.Domain.ValueObjects
{
	public class HexRgbColor : ValueObject
	{
		private HexRgbColor()
		{
		}

		public static HexRgbColor For(string hexRgbColorString)
		{
			var color = new HexRgbColor();

			try
			{
				color.Red = byte.Parse(hexRgbColorString.Substring(0, 2), NumberStyles.HexNumber);
				color.Green = byte.Parse(hexRgbColorString.Substring(2, 2), NumberStyles.HexNumber);
				color.Blue = byte.Parse(hexRgbColorString.Substring(4, 2), NumberStyles.HexNumber);
			}
			catch (Exception ex)
			{
				throw new HexRgbColorInvalidException(hexRgbColorString, ex);
			}

			return color;
		}

		public byte Red { get; private set; }
		public byte Green { get; private set; }
		public byte Blue { get; private set; }

		public static implicit operator string(HexRgbColor color)
		{
			return color.ToString();
		}

		public static explicit operator HexRgbColor(string hexRgbColorString)
		{
			return For(hexRgbColorString);
		}

		public override string ToString()
		{
			return $"{Red:X2}{Green:X2}{Blue:X2}";
		}

		protected override IEnumerable<object> GetAtomicValues()
		{
			yield return Red;
			yield return Green;
			yield return Blue;
		}
	}
}