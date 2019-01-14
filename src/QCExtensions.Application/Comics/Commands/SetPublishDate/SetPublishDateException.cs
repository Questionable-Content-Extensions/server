using System;

namespace QCExtensions.Application.Comics.Commands.SetPublishDate
{
	public class SetPublishDateException : Exception
	{
		public SetPublishDateException(string message) : base(message) { }
	}
}
