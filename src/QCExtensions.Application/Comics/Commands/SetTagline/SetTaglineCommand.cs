namespace QCExtensions.Application.Comics.Commands.SetTagline
{
	public class SetTaglineCommand : RequestWithToken
	{
		public int ComicId { get; set; }
		public string Tagline { get; set; }
	}
}
