using System;

namespace QCExtensions.Application.Comics.Commands.SetTitle
{
	public class SetTitleException : Exception
	{
		public SetTitleException(string message) : base(message) { }
	}
}
