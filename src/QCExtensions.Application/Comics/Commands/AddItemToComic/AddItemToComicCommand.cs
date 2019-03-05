using QCExtensions.Application.Interfaces;

namespace QCExtensions.Application.Comics.Commands.AddItemToComic
{
	public class AddItemToComicCommand : RequestWithToken
	{
		public const int CreateNewItemId = -1;

		public override Permission RequiredPermissions => Permission.CanAddItemToComic;

		public int ComicId { get; set; }
		public int ItemId { get; set; }

		public string NewItemType { get; set; }
		public string NewItemName { get; set; }
	}
}
