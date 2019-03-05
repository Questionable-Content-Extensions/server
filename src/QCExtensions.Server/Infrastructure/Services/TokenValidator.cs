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

		public async Task<bool> HasPermissionsAsync(Guid token, Permission permissions)
		{
			var tokenEntity = await _context.Tokens.FirstOrDefaultAsync(t => t.Id == token);
			if ((permissions & Permission.CanAddImageToItem) == Permission.CanAddImageToItem && !tokenEntity.CanAddImageToItem)
			{
				return false;
			}
			if ((permissions & Permission.CanAddItemToComic) == Permission.CanAddItemToComic && !tokenEntity.CanAddItemToComic)
			{
				return false;
			}
			if ((permissions & Permission.CanChangeComicData) == Permission.CanChangeComicData && !tokenEntity.CanChangeComicData)
			{
				return false;
			}
			if ((permissions & Permission.CanChangeItemData) == Permission.CanChangeItemData && !tokenEntity.CanChangeItemData)
			{
				return false;
			}
			if ((permissions & Permission.CanRemoveImageFromItem) == Permission.CanRemoveImageFromItem && !tokenEntity.CanRemoveImageFromItem)
			{
				return false;
			}
			if ((permissions & Permission.CanRemoveItemFromComic) == Permission.CanRemoveItemFromComic && !tokenEntity.CanRemoveItemFromComic)
			{
				return false;
			}
			return true;
		}
	}
}