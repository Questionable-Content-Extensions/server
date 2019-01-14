using System;

namespace QCExtensions.Application.Exceptions
{
	public class InvalidTokenException : Exception
	{
		public InvalidTokenException(Guid? token)
		: base($"Token '{token?.ToString() ?? "none"}' is not valid")
		{
			Token = token;
		}

		public Guid? Token { get; }
	}
}