using System;

namespace QCExtensions.Application.Exceptions
{
	public class ItemDoesNotExistException : Exception
	{
		public ItemDoesNotExistException() { }
		public ItemDoesNotExistException(string message) : base(message) { }
	}
}
