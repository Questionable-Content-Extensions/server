using System;
using MediatR;

namespace QCExtensions.Application.Interfaces
{
	public interface IRequestWithToken
	{
		Guid? Token { get; }
		bool AllowInvalidToken { get; }
		bool IsValidToken { set; }
	}
}