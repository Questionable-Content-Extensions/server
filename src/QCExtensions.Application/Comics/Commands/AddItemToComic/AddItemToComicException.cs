using System;

namespace QCExtensions.Application.Comics.Commands.AddItemToComic
{
	public class AddItemToComicException : Exception
	{
		public AddItemToComicException(string message) : base(message) { }
	}
}
