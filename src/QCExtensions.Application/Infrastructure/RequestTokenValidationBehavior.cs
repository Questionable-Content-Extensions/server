using MediatR;
using QCExtensions.Application.Interfaces;
using System;
using System.Threading;
using System.Threading.Tasks;

namespace QCExtensions.Application.Infrastructure
{
	public class RequestTokenValidationBehavior<TRequest, TResponse> : IPipelineBehavior<TRequest, TResponse>
		where TRequest : IRequest<TResponse>
	{
		private readonly ITokenValidator _tokenValidator;

		public RequestTokenValidationBehavior(ITokenValidator tokenValidator)
		{
			_tokenValidator = tokenValidator;
		}

		public async Task<TResponse> Handle(TRequest request, CancellationToken cancellationToken, RequestHandlerDelegate<TResponse> next)
		{
			if (request is IRequestWithToken requestWithToken)
			{
				if (!requestWithToken.Token.HasValue || !await _tokenValidator.IsValidAsync(requestWithToken.Token.Value))
				{
					if (!requestWithToken.AllowInvalidToken)
					{
						throw new Exceptions.InvalidTokenException(requestWithToken.Token);
					}
					requestWithToken.IsValidToken = false;
				}
				else
				{
					requestWithToken.IsValidToken = true;

					if (!await _tokenValidator.HasPermissionsAsync(requestWithToken.Token.Value, requestWithToken.RequiredPermissions))
					{
						throw new Exceptions.MissingPermissionsException(requestWithToken.Token.Value, requestWithToken.RequiredPermissions);
					}
				}
			}

			return await next();
		}
	}
}