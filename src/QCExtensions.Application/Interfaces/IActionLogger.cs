using System;
using System.Threading;
using System.Threading.Tasks;

namespace QCExtensions.Application.Interfaces
{
	public interface IActionLogger
	{
		Task LogAsync(Guid token, string action, bool saveChanges = true, CancellationToken cancellationToken = default(CancellationToken));
	}
}