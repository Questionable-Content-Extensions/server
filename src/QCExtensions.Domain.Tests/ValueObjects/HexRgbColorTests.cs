using QCExtensions.Domain.Exceptions;
using QCExtensions.Domain.ValueObjects;
using System;
using Xunit;

namespace QCExtensions.Domain.Tests.ValueObjects
{
	public class HexRgbColorTests
	{
		private const string KnownColorValue = "0A0B0C";
		private const string InvalidColorValue = "COLORS";
		private const byte KnownRedValue = 0xA;
		private const byte KnownGreenValue = 0xB;
		private const byte KnownBlueValue = 0xC;

		[Fact]
		public void ShouldParseCorrectValueForRed()
		{
			var color = HexRgbColor.For(KnownColorValue);

			Assert.Equal(KnownRedValue, color.Red);
		}

		[Fact]
		public void ShouldParseCorrectValueForGreen()
		{
			var color = HexRgbColor.For(KnownColorValue);

			Assert.Equal(KnownGreenValue, color.Green);
		}

		[Fact]
		public void ShouldParseCorrectValueForBlue()
		{
			var color = HexRgbColor.For(KnownColorValue);

			Assert.Equal(KnownBlueValue, color.Blue);
		}

		[Fact]
		public void ToStringReturnsCorrectFormat()
		{
			var color = HexRgbColor.For(KnownColorValue);

			Assert.Equal(KnownColorValue, color.ToString());
		}

		[Fact]
		public void ImplicitConversionToStringResultsInCorrectString()
		{
			var color = HexRgbColor.For(KnownColorValue);

			string result = color;

			Assert.Equal(KnownColorValue, result);
		}

		[Fact]
		public void ExplicitConversionFromStringSetsColorValues()
		{
			var color = (HexRgbColor)KnownColorValue;

			Assert.Equal(KnownRedValue, color.Red);
			Assert.Equal(KnownGreenValue, color.Green);
			Assert.Equal(KnownBlueValue, color.Blue);
		}

		[Fact]
		public void ShouldThrowAdAccountInvalidExceptionForInvalidAdAccount()
		{
			Assert.Throws<HexRgbColorInvalidException>(() => (HexRgbColor)InvalidColorValue);
		}
	}
}
