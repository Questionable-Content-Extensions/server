using QCExtensions.Application.Interfaces;

namespace QCExtensions.Application.Comics.Commands.RemoveItemFromComic
{
	public class RemoveItemFromComicCommand : RequestWithToken
	{
		public override Permission RequiredPermissions => Permission.CanRemoveItemFromComic;
		
		public int ComicId { get; set; }
		public int ItemId { get; set; }
	}
}
