namespace QCExtensions.Application.Comics.Commands.RemoveItemFromComic
{
	public class RemoveItemFromComicCommand : RequestWithToken
	{
		public int ComicId { get; set; }
		public int ItemId { get; set; }
	}
}
