using Microsoft.EntityFrameworkCore;
using QCExtensions.Application.Interfaces;
using QCExtensions.Domain.Entities;
using System;
using System.Threading.Tasks;

namespace QCExtensions.Server.Infrastructure.Services
{
	public class TokenValidator : ITokenValidator
	{
		private DomainDbContext _context;

		public TokenValidator(DomainDbContext context)
		{
			_context = context;
		}

		public async Task<bool> IsValidAsync(Guid token)
		{
			return token != Guid.Empty && await _context.Tokens.AnyAsync(t => t.Id == token);
		}
	}
}