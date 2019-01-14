using System;

namespace QCExtensions.Application.Comics.Commands.SetPublishDate
{
	public class SetPublishDateCommand : RequestWithToken
	{
		public int ComicId { get; set; }
		public DateTime PublishDate { get; set; }
		public bool IsAccuratePublishDate { get; set; }
	}
}
