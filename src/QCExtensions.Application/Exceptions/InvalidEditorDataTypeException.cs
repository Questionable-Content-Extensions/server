using System;

namespace QCExtensions.Application.Exceptions
{
	public class InvalidEditorDataTypeException : Exception
	{
		public InvalidEditorDataTypeException(string type)
		: base($"Encountered invalid navigation type '{type}'")
		{
		}
	}
}
