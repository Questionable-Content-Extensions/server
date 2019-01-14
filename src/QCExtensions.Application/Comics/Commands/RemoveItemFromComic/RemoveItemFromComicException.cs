using System;

namespace QCExtensions.Application.Comics.Commands.RemoveItemFromComic
{
	public class RemoveItemFromComicException : Exception
	{
		public RemoveItemFromComicException(string message) : base(message) { }
	}
}
