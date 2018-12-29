using System;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;
using QCExtensions.Server.Models;

namespace QCExtensions.Server.Infrastructure.Services
{
	public interface ITokenHandler
	{
		Task<bool> IsValidAsync(Guid token);
	}

	public class TokenHandler : ITokenHandler
	{
		private ApplicationDbContext _applicationDbContext;

		public TokenHandler(ApplicationDbContext applicationDbContext)
		{
			_applicationDbContext = applicationDbContext;
		}

		public async Task<bool> IsValidAsync(Guid token)
		{
			return await _applicationDbContext.Tokens.SingleOrDefaultAsync(t => t.Id == token) != null;
		}
	}
}