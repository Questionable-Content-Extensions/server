using System;
using QCExtensions.Application.Interfaces;

namespace QCExtensions.Application.Comics.Commands.SetPublishDate
{
	public class SetPublishDateCommand : RequestWithToken
	{
		public override Permission RequiredPermissions => Permission.CanChangeComicData;

		public int ComicId { get; set; }
		public DateTime PublishDate { get; set; }
		public bool IsAccuratePublishDate { get; set; }
	}
}
