using System;
using QCExtensions.Application.Interfaces;

namespace QCExtensions.Application.Exceptions
{
	public class MissingPermissionsException : Exception
	{
		public MissingPermissionsException(Guid token, Permission requiredPermissions)
		: base($"Token '{token}' does not have permission to perform the requested operation")
		{
			Token = token;
			RequiredPermissions = requiredPermissions;
		}

		public Guid Token { get; }
		public Permission RequiredPermissions { get; }
	}
}
