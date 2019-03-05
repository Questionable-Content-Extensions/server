using System;
using MediatR;

namespace QCExtensions.Application.Interfaces
{
	[Flags]
	public enum Permission
	{
		None = 0x0,
		CanAddItemToComic = 0x01,
		CanRemoveItemFromComic = 0x02,
		CanChangeComicData = 0x04,
		CanAddImageToItem = 0x08,
		CanRemoveImageFromItem = 0x10,
		CanChangeItemData = 0x20,
	}

	public interface IRequestWithToken
	{
		Guid? Token { get; }
		bool AllowInvalidToken { get; }
		bool IsValidToken { set; }
		Permission RequiredPermissions { get; }
	}
}