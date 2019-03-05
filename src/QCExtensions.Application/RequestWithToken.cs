using System;
using MediatR;
using QCExtensions.Application.Interfaces;

namespace QCExtensions.Application
{
	public abstract class RequestWithToken<TResponse> : IRequest<TResponse>, IRequestWithToken
	{
		public Guid? Token { get; set; }
		public bool IsValidToken { get; set; }

		public virtual bool AllowInvalidToken => false;

		public virtual Permission RequiredPermissions => Permission.None;
	}

	public abstract class RequestWithToken : RequestWithToken<Unit>, IRequest
	{
	}
}