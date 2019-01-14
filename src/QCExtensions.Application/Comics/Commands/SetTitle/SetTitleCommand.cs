namespace QCExtensions.Application.Comics.Commands.SetTitle
{
	public class SetTitleCommand : RequestWithToken
	{
		public int ComicId { get; set; }
		public string Title { get; set; }
	}
}
